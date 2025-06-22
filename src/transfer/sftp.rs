// sftp.rs

use crate::{
    Authentication, AuthenticationMethod, TransferProfile, error::AppError,
    transfer::protocol::TransferProtocolHandler,
};
use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, error, info};
use ssh2::{Session, Sftp};
use ssh2_config::{ParseRule, SshConfig};
use std::{
    fs::File,
    io::copy,
    net::TcpStream,
    path::{Path, PathBuf},
};

pub struct SftpHandler;

#[async_trait::async_trait]
impl TransferProtocolHandler for SftpHandler {
    async fn send(&self, profile: &TransferProfile) -> Result<()> {
        info!(
            "Attempting to send file from '{}' to SFTP destination '{}'@{}:{}{}",
            profile.source.path,
            profile
                .destination
                .authentication
                .as_ref()
                .map_or("unknown", |a| a.username.as_str()),
            profile.destination.host.as_deref().unwrap_or("localhost"),
            profile.destination.port.unwrap_or(22),
            profile.destination.path
        );
        let sftp = connect_sftp_and_authenticate(
            profile.destination.authentication.as_ref(),
            profile.destination.host.as_deref(),
            profile.destination.port,
        )?;

        transfer_file_sftp(
            &sftp,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            true, // upload
        )
    }

    async fn receive(&self, profile: &TransferProfile) -> Result<()> {
        info!(
            "Attempting to receive file from SFTP source '{}'@{}:{}{} to local '{}'",
            profile
                .source
                .authentication
                .as_ref()
                .map_or("unknown", |a| a.username.as_str()),
            profile.source.host.as_deref().unwrap_or("localhost"),
            profile.source.port.unwrap_or(22),
            profile.source.path,
            profile.destination.path
        );
        let sftp = connect_sftp_and_authenticate(
            profile.source.authentication.as_ref(),
            profile.source.host.as_deref(),
            profile.source.port,
        )?;

        transfer_file_sftp(
            &sftp,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            false, // download
        )
    }
}

// 認証情報（秘密鍵）のパスを取得する関数
// Function to get the path of authentication information (private key)
fn get_private_key_path(auth: &Authentication) -> Result<String, AppError> {
    if let Some(key_ref) = &auth.private_key_ref {
        match auth.method {
            AuthenticationMethod::EnvKey => Ok(std::env::var(key_ref)
                .map_err(|_| AppError::EnvVarNotFound(key_ref.clone()))
                .context(format!(
                    "Failed to get private key path from environment variable '{}'",
                    key_ref
                ))?),
            AuthenticationMethod::PrivateKey => Ok(key_ref.clone()),
            _ => Err(AppError::AuthenticationFailed(
                "Unsupported authentication method for get_private_key_path".into(),
            )),
        }
    } else {
        Err(AppError::MissingPrivateKeyReference)
    }
}

fn connect_sftp_and_authenticate(
    auth: Option<&Authentication>,
    host_opt: Option<&str>,
    port_opt: Option<u16>,
) -> Result<Sftp> {
    let auth = auth.ok_or(AppError::AuthenticationFailed("Missing auth".into()))?;

    let mut host = host_opt.unwrap_or("localhost").to_string();
    let mut port = port_opt.unwrap_or(22);
    let mut username = auth.username.clone();
    let mut private_key_path: Option<PathBuf> = None;
    let mut password: Option<String> = None;

    info!("Connecting to SFTP server: {}@{}:{}", username, host, port);

    match auth.method {
        AuthenticationMethod::Password => {
            password = auth.password_ref.as_ref().map(|ref_name| {
                std::env::var(ref_name).unwrap_or_else(|_| {
                    eprintln!("Warning: env var '{}' not found.", ref_name);
                    String::new()
                })
            });
        }
        AuthenticationMethod::PrivateKey | AuthenticationMethod::EnvKey => {
            let key_str = get_private_key_path(auth)?;
            private_key_path = Some(PathBuf::from(key_str));
        }
        AuthenticationMethod::SshConfig => {
            let alias = auth.ssh_config_alias.as_ref().ok_or(AppError::Validation(
                "sshConfigAlias is required for SshConfig method".to_string(),
            ))?;

            let ssh_config = SshConfig::parse_default_file(ParseRule::STRICT)?;
            let host_config = ssh_config.query(alias);

            host = host_config.host_name.unwrap_or_else(|| host.clone());
            port = host_config.port.unwrap_or(port);

            if let Some(user_from_config) = host_config.user {
                username = user_from_config;
            }

            if let Some(identity_files) = &host_config.identity_file {
                if let Some(first) = identity_files.first() {
                    let path = PathBuf::from(first);
                    let resolved = if path.is_relative() {
                        home_dir()
                            .context("No home dir found for resolving relative path")?
                            .join(&path)
                    } else {
                        path
                    };
                    debug!(
                        "Using identity file from SSH config: '{}'",
                        resolved.display()
                    );
                    private_key_path = Some(resolved);
                } else {
                    debug!("SSH config had identity_file entry but it was empty.");
                }
            } else {
                debug!("SSH config did not specify identity_file.");
            }
            info!(
                "Resolved connection details via SSH config: User={}, Host={}, Port={}",
                username, host, port
            );
        }
    }

    let tcp = TcpStream::connect((host.clone(), port))
        .with_context(|| format!("Failed to connect to {}:{}", host, port))?;
    info!("TCP connection established to {}:{}", host, port);

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake().context("SSH handshake failed")?;
    info!("SSH handshake successful.");

    if let Some(path) = private_key_path {
        info!(
            "Attempting public key authentication with key: '{}'",
            path.display()
        );
        sess.userauth_pubkey_file(&username, None, &path, None)
            .with_context(|| {
                format!(
                    "Private key authentication failed for user '{}' using key '{}'",
                    username,
                    path.display()
                )
            })?;
    } else if let Some(pw) = password {
        info!("Attempting password authentication.");
        sess.userauth_password(&username, &pw)
            .with_context(|| format!("Password authentication failed for user '{}'", username))?;
    } else {
        error!(
            "No suitable authentication method could be resolved for user '{}'.",
            username
        );
        return Err(AppError::AuthenticationFailed("No auth method resolved".into()).into());
    }

    if !sess.authenticated() {
        error!(
            "SFTP authentication failed for user '{}'. Session not authenticated.",
            username
        );
        return Err(AppError::AuthenticationFailed("SFTP authentication failed".into()).into());
    }
    info!("SFTP authentication successful for user: '{}'.", username);

    Ok(sess.sftp()?)
}

fn transfer_file_sftp(sftp: &Sftp, src: &PathBuf, dst: &PathBuf, upload: bool) -> Result<()> {
    if upload {
        info!(
            "Attempting to upload file from '{}' to remote path '{}'",
            src.display(),
            dst.display()
        );
        let mut local_file = File::open(src).with_context(|| {
            format!(
                "Failed to open local source file for upload: '{}'",
                src.display()
            )
        })?;
        let mut remote_file = sftp.create(Path::new(dst)).with_context(|| {
            format!(
                "Failed to create remote destination file for upload: '{}'",
                dst.display()
            )
        })?;
        copy(&mut local_file, &mut remote_file).with_context(|| {
            format!(
                "Failed to copy data during upload from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
        info!(
            "Successfully uploaded file from '{}' to '{}'",
            src.display(),
            dst.display()
        );
    } else {
        info!(
            "Attempting to download file from remote path '{}' to local path '{}'",
            src.display(),
            dst.display()
        );
        let mut remote_file = sftp.open(Path::new(src)).with_context(|| {
            format!(
                "Failed to open remote source file for download: '{}'",
                src.display()
            )
        })?;
        let mut local_file = File::create(dst).with_context(|| {
            format!(
                "Failed to create local destination file for download: '{}'",
                dst.display()
            )
        })?;
        copy(&mut remote_file, &mut local_file).with_context(|| {
            format!(
                "Failed to copy data during download from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
        info!(
            "Successfully downloaded file from '{}' to '{}'",
            src.display(),
            dst.display()
        );
    }

    Ok(())
}

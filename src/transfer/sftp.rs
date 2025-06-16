// sftp.rs

use crate::{
    Authentication, AuthenticationMethod, TransferProfile, error::AppError,
    transfer::protocol::TransferProtocolHandler,
};
use anyhow::{Context, Result};
use dirs::home_dir;
use ssh2::{Session, Sftp};
use ssh2_config::{ParseRule, SshConfig};
use std::{
    fs::File,
    io::copy,
    net::TcpStream,
    path::{Path, PathBuf},
};

pub struct SftpHandler;

impl TransferProtocolHandler for SftpHandler {
    fn send(&self, profile: &TransferProfile) -> Result<()> {
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

    fn receive(&self, profile: &TransferProfile) -> Result<()> {
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

// 🔐 認証情報（秘密鍵）のパスを取得する関数（修正なし）
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
                        home_dir().context("No home dir")?.join(path)
                    } else {
                        path
                    };
                    private_key_path = Some(resolved);
                }
            }
        }
    }

    let tcp = TcpStream::connect((host.clone(), port))
        .with_context(|| format!("Failed to connect to {}:{}", host, port))?;

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    if let Some(path) = private_key_path {
        sess.userauth_pubkey_file(&username, None, &path, None)
            .with_context(|| format!("Private key auth failed for {}", username))?;
    } else if let Some(pw) = password {
        sess.userauth_password(&username, &pw)
            .with_context(|| format!("Password auth failed for {}", username))?;
    } else {
        return Err(AppError::AuthenticationFailed("No auth method resolved".into()).into());
    }

    if !sess.authenticated() {
        return Err(AppError::AuthenticationFailed("SFTP authentication failed".into()).into());
    }

    Ok(sess.sftp()?)
}

fn transfer_file_sftp(sftp: &Sftp, src: &PathBuf, dst: &PathBuf, upload: bool) -> Result<()> {
    if upload {
        let mut local_file = File::open(src)?;
        let mut remote_file = sftp.create(Path::new(dst))?;
        copy(&mut local_file, &mut remote_file)?;
    } else {
        let mut remote_file = sftp.open(Path::new(src))?;
        let mut local_file = File::create(dst)?;
        copy(&mut remote_file, &mut local_file)?;
    }

    Ok(())
}

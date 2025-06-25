use crate::{
    Authentication, AuthenticationMethod, TransferProfile, error::AppError,
    transfer::protocol::TransferProtocolHandler,
    util::get_private_key_path,
};
use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, error, info};
use ssh2::Session;
use ssh2_config::{ParseRule, SshConfig};
use std::{
    fs::File, io::{Read, Write}, net::TcpStream, path::{Path, PathBuf}
};

pub struct ScpHandler;

#[async_trait::async_trait]
impl TransferProtocolHandler for ScpHandler {
    async fn send(&self, profile: &TransferProfile) -> Result<()> {
        info!(
            "Attempting to send file from '{}' to SCP destination '{}'@{}:{}{}",
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
        let session = connect_scp_and_authenticate(
            profile.destination.authentication.as_ref(),
            profile.destination.host.as_deref(),
            profile.destination.port,
        )?;

        let mut remote_file = session.scp_send(Path::new(&profile.destination.path), 0o644, 10, None)?;
        // Permissions on sent files are set to 0o644 (owner: read/write, group: read, other: read).
        // The file transfer timeout is set to 10 seconds.
        // No special callback processing is performed during file transfer.
        println!("start writing");
        remote_file.write_all(b"1234567890").unwrap();
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();

        println!("end");
        Ok(())
    }

    async fn receive(&self, profile: &TransferProfile) -> Result<()> {
        info!(
            "Attempting to receive file from SCP source '{}'@{}:{}{} to local '{}'",
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

        let session = connect_scp_and_authenticate(
            profile.source.authentication.as_ref(),
            profile.source.host.as_deref(),
            profile.source.port,
        )?;

        let (mut remote_file, _) = session.scp_recv(Path::new(&profile.source.path))?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        // Write contents to local file
        let local_path = PathBuf::from(&profile.destination.path);
        let mut file = File::create(&local_path)?;
        file.write_all(&contents)?;

        Ok(())
    }
}

fn connect_scp_and_authenticate(
    auth: Option<&Authentication>,
    host_opt: Option<&str>,
    port_opt: Option<u16>,
) -> Result<Session> {
    let auth = auth.ok_or(AppError::AuthenticationFailed("Missing auth".into()))?;

    let mut host = host_opt.unwrap_or("localhost").to_string();
    let mut port = port_opt.unwrap_or(22);
    let mut username = auth.username.clone();
    let mut private_key_path: Option<PathBuf> = None;
    let mut password: Option<String> = None;

    info!("Connecting to SCP server: {}@{}:{}", username, host, port);

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
            "SCP authentication failed for user '{}'. Session not authenticated.",
            username
        );
        return Err(AppError::AuthenticationFailed("SCP authentication failed".into()).into());
    }
    info!("SCP authentication successful for user: '{}'.", username);

    Ok(sess)
}

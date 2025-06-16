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
        let auth =
            profile
                .destination
                .authentication
                .as_ref()
                .ok_or(AppError::AuthenticationFailed(
                    "Missing authentication info".into(),
                ))?;

        let mut host: String = profile
            .destination
            .host
            .clone()
            .unwrap_or_else(|| "localhost".into());

        let mut port = profile.destination.port.unwrap_or(22);
        let mut username_owned = auth.username.clone();

        let mut private_key_path: Option<PathBuf> = None;
        let mut password: Option<String> = None;

        match auth.method {
            AuthenticationMethod::Password => {
                password = auth.password_ref.as_ref().map(|ref_name| {
                    std::env::var(ref_name).unwrap_or_else(|_| {
                        eprintln!(
                            "Warning: Password environment variable '{}' not found.",
                            ref_name
                        );
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

                let ssh_config = SshConfig::parse_default_file(ParseRule::STRICT)
                    .context("Failed to read SSH configuration")?;
                let host_config = ssh_config.query(alias);

                host = host_config.host_name.unwrap_or_else(|| host.clone());
                port = host_config.port.unwrap_or(port);

                if let Some(user_from_config) = host_config.user {
                    username_owned = user_from_config;
                }

                if let Some(identity_files) = &host_config.identity_file {
                    if let Some(first) = identity_files.first() {
                        let current_path = PathBuf::from(first);
                        let resolved_path = if current_path.is_relative() {
                            home_dir()
                                .context("Failed to get home directory")?
                                .join(current_path)
                        } else {
                            current_path
                        };
                        private_key_path = Some(resolved_path);
                    }
                }
            }
        }

        // セッション開始
        let tcp = TcpStream::connect((host.clone(), port))
            .with_context(|| format!("Failed to connect to {}:{}", host, port))?;

        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        // 認証処理
        if let Some(path) = private_key_path {
            sess.userauth_pubkey_file(username_owned.as_str(), None, &path, None)
                .with_context(|| {
                    format!(
                        "Failed to authenticate with private key for user '{}' at {:?}",
                        username_owned, path
                    )
                })?;
        } else if let Some(p) = password {
            sess.userauth_password(username_owned.as_str(), &p)
                .with_context(|| {
                    format!(
                        "Failed to authenticate with password for user '{}'",
                        username_owned
                    )
                })?;
        } else {
            return Err(AppError::AuthenticationFailed(
                "No valid authentication method found.".into(),
            )
            .into());
        }

        if !sess.authenticated() {
            return Err(AppError::AuthenticationFailed("SFTP authentication failed".into()).into());
        }

        let sftp: Sftp = sess.sftp()?;

        // ファイル転送
        let mut local_file =
            File::open(&profile.source.path).context("Failed to open local file")?;
        let mut remote_file = sftp
            .create(Path::new(&profile.destination.path))
            .context("Failed to create remote file")?;

        copy(&mut local_file, &mut remote_file).context("Failed to copy file")?;

        Ok(())
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

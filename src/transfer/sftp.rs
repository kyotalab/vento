use crate::{
    Authentication, AuthenticationMethod, TransferProfile, error::AppError,
    transfer::protocol::TransferProtocolHandler,
};
use anyhow::{Context, Result};
use ssh2::Session;
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

        let private_key_str = get_private_key_from_env(auth)?;
        let private_key_path = PathBuf::from(private_key_str);

        // SSHセッションを開始
        let tcp = TcpStream::connect((
            profile.destination.host.as_deref().unwrap_or("localhost"),
            profile.destination.port.unwrap_or(22),
        ))?;

        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        sess.userauth_pubkey_file(&auth.username, None, &private_key_path, None)?;

        if !sess.authenticated() {
            return Err(AppError::AuthenticationFailed("SFTP authentication failed".into()).into());
        }

        let sftp = sess.sftp()?;

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

// 🔐 認証情報（秘密鍵）を環境変数から取得
fn get_private_key_from_env(auth: &Authentication) -> Result<String, AppError> {
    if matches!(
        auth.method,
        AuthenticationMethod::PrivateKey | AuthenticationMethod::EnvKey
    ) {
        if let Some(key_ref) = &auth.private_key_ref {
            let private_key = std::env::var(key_ref)
                .map_err(|_| AppError::EnvVarNotFound(key_ref.clone()))
                .context(format!(
                    "Failed to get private key from environment variable '{}'",
                    key_ref
                ))?;
            return Ok(private_key);
        } else {
            return Err(AppError::MissingPrivateKeyReference);
        }
    }

    Err(AppError::AuthenticationFailed(
        "Unsupported authentication method".into(),
    ))
}

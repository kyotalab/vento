use crate::{
    Authentication, AuthenticationMethod, error::AppError,
};
use anyhow::{Context, Result};
// 認証情報（秘密鍵）のパスを取得する関数
// Function to get the path of authentication information (private key)
pub fn get_private_key_path(auth: &Authentication) -> Result<String, AppError> {
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

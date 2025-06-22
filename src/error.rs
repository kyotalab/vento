use cron::error::Error as CronError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    // std::io::Error から自動変換
    // Automatic conversion from std::io::Error
    Io(#[from] std::io::Error),

    #[error("YAML deserialization error: {0}")]
    // serde_yaml::Error から自動変換
    // Automatic conversion from serde_yaml::Error
    Yaml(#[from] serde_yaml::Error),

    #[error("Validation error: {0}")]
    // 一般的なバリデーションエラーメッセージ
    // Common validation error messages
    Validation(String),

    #[error("Environment variable '{0}' not found or invalid.")]
    EnvVarNotFound(String),

    #[error("Authentication failed: {0}")]
    // 認証失敗の具体的な理由
    // The specific reason for the authentication failure
    AuthenticationFailed(String),

    #[error("Schedule is required when trigger type is 'schedule'.")]
    // `schedule`がNoneの場合のエラー
    // Error if `schedule` is None
    MissingSchedule,

    #[error("Invalid cron schedule '{expression}': {source}")]
    // cron::error::Error を使用
    // Use cron::error::Error
    InvalidCronSchedule {
        expression: String,
        #[source]
        source: CronError,
    },

    #[error("Private key reference is missing in the authentication config.")]
    MissingPrivateKeyReference,
    //... 他の具体的なエラー
    // Other specific errors
}

// anyhow::Error から AppError::Validation への変換は既存のままでOK
// The conversion from anyhow::Error to AppError::Validation can be left as is.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Validation(err.to_string())
    }
}

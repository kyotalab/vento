use cron::error::Error as CronError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error), // std::io::Error から自動変換

    #[error("YAML deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error), // serde_yaml::Error から自動変換

    #[error("Validation error: {0}")]
    Validation(String), // 一般的なバリデーションエラーメッセージ

    #[error("Environment variable '{0}' not found or invalid.")]
    EnvVarNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String), // 認証失敗の具体的な理由

    #[error("Schedule is required when trigger type is 'schedule'.")]
    MissingSchedule, // `schedule`がNoneの場合のエラー

    #[error("Invalid cron schedule '{expression}': {source}")]
    InvalidCronSchedule {
        expression: String,
        #[source]
        source: CronError, // cron::error::Error を使用
    },

    #[error("Private key reference is missing in the authentication config.")]
    MissingPrivateKeyReference,
    //... 他の具体的なエラー
}

// anyhow::Error から AppError::Validation への変換は既存のままでOK
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Validation(err.to_string())
    }
}

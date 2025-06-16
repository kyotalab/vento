use std::str::FromStr as _;

use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize};

use crate::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub transfer_profiles: Vec<TransferProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProfile {
    pub profile_id: String,
    pub description: Option<String>,
    pub source: Source,
    pub destination: Destination,
    pub transfer_protocol: TransferProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(rename = "type")] // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    pub kind: SourceType,
    pub path: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub authentication: Option<Authentication>,
    pub trigger: Trigger,
}

impl Source {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.kind == SourceType::Sftp {
            if self.host.is_none() {
                return Err(AppError::Validation(
                    "SFTP source requires 'host'".to_string(),
                ));
            }
            if self.port.is_none() {
                return Err(AppError::Validation(
                    "SFTP source requires 'port'".to_string(),
                ));
            }
            if self.authentication.is_none() {
                return Err(AppError::Validation(
                    "SFTP source requires 'authentication'".to_string(),
                ));
            } else {
                self.authentication.as_ref().unwrap().validate()?;
            }
        }
        if self.trigger.kind == TriggerType::Schedule {
            if let Some(s) = &self.trigger.schedule {
                s.parse::<cron::Schedule>()
                    .map_err(|e| AppError::InvalidCronSchedule {
                        expression: s.clone(),
                        source: e,
                    })?;
            } else {
                return Err(AppError::Validation(
                    "Schedule is required when trigger type is 'schedule'".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Authentication {
    pub fn validate(&self) -> Result<(), AppError> {
        match self.method {
            AuthenticationMethod::Password => {
                if self.password_ref.is_none() {
                    return Err(AppError::AuthenticationFailed(
                        "Password authentication requires 'passwordRef'".to_string(),
                    ));
                }
            }
            AuthenticationMethod::PrivateKey | AuthenticationMethod::EnvKey => {
                if self.private_key_ref.is_none() {
                    return Err(AppError::AuthenticationFailed(
                        "Private key authentication requires 'privateKeyRef'".to_string(),
                    ));
                }
            } // Manualなど、他の認証方法があればここに追加
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Local,
    Sftp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authentication {
    pub method: AuthenticationMethod,
    pub username: String,
    pub password_ref: Option<String>,
    pub private_key_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod {
    Password,
    PrivateKey,
    EnvKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    #[serde(rename = "type")] // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    pub kind: TriggerType,
    pub schedule: Option<String>,
}

impl Trigger {
    pub fn validate(&self) -> Result<(), Error> {
        // Result<(), anyhow::Error> を返す
        if self.kind == TriggerType::Schedule {
            // schedule が None の場合は AppError::MissingSchedule を返す
            let schedule_expression = self
                .schedule
                .as_ref()
                .ok_or_else(|| AppError::MissingSchedule)?; // `?` で早期リターン

            // Cron式のパースを試みる
            // パースに失敗した場合は AppError::InvalidCronSchedule を返す
            cron::Schedule::from_str(schedule_expression).map_err(|e| {
                AppError::InvalidCronSchedule {
                    expression: schedule_expression.clone(), // パースできなかった文字列
                    source: e,                               // cron::error::Error
                }
            })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Manual,
    Schedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    #[serde(rename = "type")] // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    pub kind: DestinationType,
    pub path: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub authentication: Option<Authentication>,
}

impl Destination {
    pub fn validate(&self) -> Result<(), Error> {
        if self.kind == DestinationType::Sftp {
            if self.host.is_none() {
                return Err(anyhow!("SFTP destination requires 'host'"));
            }
            if self.port.is_none() {
                return Err(anyhow!("SFTP destination requires 'port'"));
            }
            if self.authentication.is_none() {
                return Err(anyhow!("SFTP destination requires 'authentication'"));
            } else {
                self.authentication.as_ref().unwrap().validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DestinationType {
    Local,
    Sftp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProtocol {
    pub protocol: ProtocolType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProtocolType {
    Sftp,
}

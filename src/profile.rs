use std::{fs, path::Path, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{validate_ascii, validate_cross_platform_path, AppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub transfer_profiles: Vec<TransferProfile>,
}

impl Profile {
    pub fn load_profiles(path: &Path) -> Result<Profile> {
        let yaml = fs::read_to_string(path)?;
        let profiles: Profile = serde_yaml::from_str(&yaml)?;
        Ok(profiles)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransferProfile {
    #[validate(length(min = 1, max = 32), regex(path = "*PROFILE_ID_REGEX"))]
    pub profile_id: String,

    #[validate(length(max = 128))]
    pub description: Option<String>,

    pub source: Source,
    pub destination: Destination,
    pub transfer_protocol: TransferProtocol,

    #[validate(length(min = 1, max = 256))]
    #[validate(custom(function = "validate_ascii"))]
    pub pre_transfer_command: Option<String>,

    #[validate(length(min = 1, max = 256))]
    #[validate(custom(function = "validate_ascii"))]
    pub post_transfer_command: Option<String>,

    #[validate(length(min = 1, max = 256))]
    #[validate(custom(function = "validate_ascii"))]
    pub on_error_command: Option<String>,
}

lazy_static::lazy_static! {
    static ref PROFILE_ID_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    // Map YAML 'type' keys to Rust 'kind' fields
    #[serde(rename = "type")]
    pub kind: SourceType,

    #[validate(custom(function = "validate_cross_platform_path"))]
    pub path: String,

    pub host: Option<String>,
    pub port: Option<u16>,
    pub authentication: Option<Authentication>,
    pub trigger: Trigger,
}

impl Source {
    pub fn validate(&self) -> Result<(), AppError> {
        match self.kind {
            SourceType::Local => {
                println!("local")
            }
            SourceType::Sftp => {
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
            SourceType::Scp => {
                if self.host.is_none() {
                    return Err(AppError::Validation(
                        "SCP source requires 'host'".to_string(),
                    ));
                }
                if self.port.is_none() {
                    return Err(AppError::Validation(
                        "SCP source requires 'port'".to_string(),
                    ));
                }
                if self.authentication.is_none() {
                    return Err(AppError::Validation(
                        "SCP source requires 'authentication'".to_string(),
                    ));
                } else {
                    self.authentication.as_ref().unwrap().validate()?;
                }
            }
        }

        match self.trigger.kind {
            TriggerType::Manual => {
                println!("manual")
            }
            TriggerType::Schedule => {
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
                    return Err(AppError::Validation(
                        "Private key or Env key requires 'privateKeyRef'".to_string(),
                    ));
                }
            } // Manualなど、他の認証方法があればここに追加
            AuthenticationMethod::SshConfig => {
                if self.ssh_config_alias.is_none() {
                    return Err(AppError::Validation(
                        "SSH config authentication requires 'sshConfigAlias'".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Local,
    Sftp,
    Scp,
}

impl ToString for SourceType {
    fn to_string(&self) -> String {
        match self {
            SourceType::Local => "local".into(),
            SourceType::Sftp => "sftp".into(),
            SourceType::Scp => "scp".into(),
        }
    }
}

impl FromStr for SourceType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(SourceType::Local),
            "sftp" => Ok(SourceType::Sftp),
            "scp" => Ok(SourceType::Scp),
            other => Err(format!("'{}' is not allowed", other).into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authentication {
    pub method: AuthenticationMethod,
    pub username: String,
    pub password_ref: Option<String>,
    pub private_key_ref: Option<String>,
    pub ssh_config_alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod {
    Password,
    PrivateKey,
    EnvKey,
    SshConfig,
}

impl ToString for AuthenticationMethod {
   fn to_string(&self) -> String {
        match self {
            AuthenticationMethod::Password => "password".into(),
            AuthenticationMethod::PrivateKey => "private_key".into(),
            AuthenticationMethod::EnvKey => "env_key".into(),
            AuthenticationMethod::SshConfig => "ssh_config".into(),
        }
    } 
}


impl FromStr for AuthenticationMethod {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "password" => Ok(AuthenticationMethod::Password),
            "private_key" => Ok(AuthenticationMethod::PrivateKey),
            "env_key" => Ok(AuthenticationMethod::EnvKey),
            "ssh_config" => Ok(AuthenticationMethod::SshConfig),
            other => Err(format!("'{}' is not allowed", other).into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    #[serde(rename = "type")] // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    pub kind: TriggerType,
    pub schedule: Option<String>,
}

impl Trigger {
    pub fn validate(&self) -> Result<(), AppError> {
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

impl ToString for TriggerType {
    fn to_string(&self) -> String {
        match self {
            TriggerType::Manual => "manual".into(),
            TriggerType::Schedule => "schedule".into(),
        }
    }
}

impl FromStr for TriggerType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sftp" => Ok(TriggerType::Manual),
            "scp" => Ok(TriggerType::Schedule),
            other => Err(format!("'{}' is not allowed", other).into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    #[serde(rename = "type")] // YAMLの'type'キーをRustの'kind'フィールドにマッピング
    pub kind: DestinationType,

    #[validate(custom(function = "validate_cross_platform_path"))]
    pub path: String,

    pub host: Option<String>,
    pub port: Option<u16>,
    pub authentication: Option<Authentication>,
}

impl Destination {
    pub fn validate(&self) -> Result<(), AppError> {
        match self.kind {
            DestinationType::Local => {
                println!("local")
            }
            DestinationType::Sftp => {
                if self.host.is_none() {
                    return Err(AppError::Validation(
                        "SFTP destination requires 'host'".to_string(),
                    ));
                }
                if self.port.is_none() {
                    return Err(AppError::Validation(
                        "SFTP destination requires 'port'".to_string(),
                    ));
                }
                if self.authentication.is_none() {
                    return Err(AppError::Validation(
                        "SFTP destination requires 'authentication'".to_string(),
                    ));
                } else {
                    self.authentication.as_ref().unwrap().validate()?;
                }
            }
            DestinationType::Scp => {
                if self.host.is_none() {
                    return Err(AppError::Validation(
                        "SCP destination requires 'host'".to_string(),
                    ));
                }
                if self.port.is_none() {
                    return Err(AppError::Validation(
                        "SCP destination requires 'port'".to_string(),
                    ));
                }
                if self.authentication.is_none() {
                    return Err(AppError::Validation(
                        "SCP destination requires 'authentication'".to_string(),
                    ));
                } else {
                    self.authentication.as_ref().unwrap().validate()?;
                }
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
    Scp,
}

impl ToString for DestinationType {
    fn to_string(&self) -> String {
        match self {
            DestinationType::Local => "local".into(),
            DestinationType::Sftp => "sftp".into(),
            DestinationType::Scp => "scp".into(),
        }
    }
}

impl FromStr for DestinationType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(DestinationType::Local),
            "sftp" => Ok(DestinationType::Sftp),
            "scp" => Ok(DestinationType::Scp),
            other => Err(format!("'{}' is not allowed", other).into())
        }
    }
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
    Scp,
}

impl ToString for ProtocolType {
    fn to_string(&self) -> String {
        match self {
            ProtocolType::Sftp => "SFTP".into(),
            ProtocolType::Scp => "SCP".into(),
        }
    }
}

impl FromStr for ProtocolType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sftp" => Ok(ProtocolType::Sftp),
            "scp" => Ok(ProtocolType::Scp),
            other => Err(format!("'{}' is not allowed", other).into())
        }
    }
}

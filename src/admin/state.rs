use crate::{AppConfig, Authentication, AuthenticationMethod, DestinationType, Profile, ProtocolType, SourceType, TransferProfile, TriggerType};


pub enum AdminMode {
    Profile,
    Config,
}

pub struct AdminState {
    pub mode: AdminMode,
    pub profiles: Profile,
    pub config: AppConfig,
    pub selected_index: usize,
    pub ui_state: UiState,
}

pub enum UiState {
    ListView,
    EditView(EditState),
}

pub struct EditState {
    pub input_fields: Vec<InputField>,
    pub current_fields: usize,
}

impl EditState {
    pub fn from_profile(profile: &TransferProfile) -> Self {
        let input_fields = vec![
            InputField::new("profile_id", &profile.profile_id, Some("識別用ID")),
            InputField::new("description", profile.description.as_deref().unwrap_or_default(), Some("説明 (任意)")),

            // Source
            InputField::new("source.type", &profile.source.kind.to_string(), Some("Local / Sftp / Scp")),
            InputField::new("source.path", &profile.source.path, Some("送信元パス")),
            InputField::new("source.host", profile.source.host.as_deref().unwrap_or_default(), Some("ホスト名 (任意)")),
            InputField::new("source.port", &profile.source.port.map(|p| p.to_string()).unwrap_or_default(), Some("ポート番号 (任意)")),
            InputField::new("source.trigger", &profile.source.trigger.kind.to_string(), Some("Manual / Schedule")),
            InputField::new("source.schedule", profile.source.trigger.schedule.as_deref().unwrap_or_default(), Some("cron式 (任意)")),

            InputField::new("source.auth.method", &profile.source.authentication.as_ref().map(|a| a.method.to_string()).unwrap_or_default(), Some("Password / PrivateKey / EnvKey / SshConfig")),
            InputField::new("source.auth.username", &profile.source.authentication.as_ref().map(|a| a.username.clone()).unwrap_or_default(), None),
            InputField::new("source.auth.password_ref", &profile.source.authentication.as_ref().and_then(|a| a.password_ref.clone()).unwrap_or_default(), None),
            InputField::new("source.auth.private_key_ref", &profile.source.authentication.as_ref().and_then(|a| a.private_key_ref.clone()).unwrap_or_default(), None),
            InputField::new("source.auth.ssh_config_alias", &profile.source.authentication.as_ref().and_then(|a| a.ssh_config_alias.clone()).unwrap_or_default(), None),

            // Destination
            InputField::new("destination.type", &profile.destination.kind.to_string(), Some("Local / Sftp / Scp")),
            InputField::new("destination.path", &profile.destination.path, Some("送信先パス")),
            InputField::new("destination.host", profile.destination.host.as_deref().unwrap_or_default(), Some("ホスト名 (任意)")),
            InputField::new("destination.port", &profile.destination.port.map(|p| p.to_string()).unwrap_or_default(), Some("ポート番号 (任意)")),
            InputField::new("destination.auth.method", &profile.destination.authentication.as_ref().map(|a| a.method.to_string()).unwrap_or_default(), Some("Password / PrivateKey / EnvKey / SshConfig")),
            InputField::new("destination.auth.username", &profile.destination.authentication.as_ref().map(|a| a.username.clone()).unwrap_or_default(), None),
            InputField::new("destination.auth.password_ref", &profile.destination.authentication.as_ref().and_then(|a| a.password_ref.clone()).unwrap_or_default(), None),
            InputField::new("destination.auth.private_key_ref", &profile.destination.authentication.as_ref().and_then(|a| a.private_key_ref.clone()).unwrap_or_default(), None),
            InputField::new("destination.auth.ssh_config_alias", &profile.destination.authentication.as_ref().and_then(|a| a.ssh_config_alias.clone()).unwrap_or_default(), None),

            // Transfer Settings
            InputField::new("transfer_protocol", &profile.transfer_protocol.protocol.to_string(), Some("SFTP / SCP")),
            InputField::new("pre_transfer_command", profile.pre_transfer_command.as_deref().unwrap_or_default(), Some("事前コマンド (任意)")),
            InputField::new("post_transfer_command", profile.post_transfer_command.as_deref().unwrap_or_default(), Some("事後コマンド (任意)")),
            InputField::new("on_error_command", profile.on_error_command.as_deref().unwrap_or_default(), Some("エラー時コマンド (任意)")),
        ];

        EditState {
            input_fields,
            current_fields: 0,
        }
    }

    pub fn write_back_to_profile(&self, profile: &mut TransferProfile) {
        for field in &self.input_fields {
            match field.label.as_str() {
                "profile_id" => profile.profile_id = field.value.clone(),
                "description" => {
                    profile.description = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    }
                }

                // Source
                "source.type" => profile.source.kind = field.value.parse().unwrap_or(SourceType::Local),
                "source.path" => profile.source.path = field.value.clone(),
                "source.host" => profile.source.host = if field.value.is_empty() { None } else { Some(field.value.clone()) },
                "source.port" => profile.source.port = field.value.parse().ok(),
                "source.trigger" => profile.source.trigger.kind = field.value.parse().unwrap_or(TriggerType::Manual),
                "source.schedule" => {
                    profile.source.trigger.schedule = if field.value.is_empty() {
                        None
                    } else {
                        Some(field.value.clone())
                    }
                }

                "source.auth.method" => {
                    if profile.source.authentication.is_none() {
                        profile.source.authentication = Some(Authentication {
                            method: field.value.parse().unwrap_or(AuthenticationMethod::Password),
                            username: String::new(),
                            password_ref: None,
                            private_key_ref: None,
                            ssh_config_alias: None,
                        });
                    } else if let Some(auth) = &mut profile.source.authentication {
                        auth.method = field.value.parse().unwrap_or(AuthenticationMethod::Password);
                    }
                }
                "source.auth.username" => {
                    if let Some(auth) = &mut profile.source.authentication {
                        auth.username = field.value.clone();
                    }
                }
                "source.auth.password_ref" => {
                    if let Some(auth) = &mut profile.source.authentication {
                        auth.password_ref = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }
                "source.auth.private_key_ref" => {
                    if let Some(auth) = &mut profile.source.authentication {
                        auth.private_key_ref = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }
                "source.auth.ssh_config_alias" => {
                    if let Some(auth) = &mut profile.source.authentication {
                        auth.ssh_config_alias = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }

                // Destination
                "destination.type" => profile.destination.kind = field.value.parse().unwrap_or(DestinationType::Local),
                "destination.path" => profile.destination.path = field.value.clone(),
                "destination.host" => profile.destination.host = if field.value.is_empty() { None } else { Some(field.value.clone()) },
                "destination.port" => profile.destination.port = field.value.parse().ok(),
                "destination.auth.method" => {
                    if profile.destination.authentication.is_none() {
                        profile.destination.authentication = Some(Authentication {
                            method: field.value.parse().unwrap_or(AuthenticationMethod::Password),
                            username: String::new(),
                            password_ref: None,
                            private_key_ref: None,
                            ssh_config_alias: None,
                        });
                    } else if let Some(auth) = &mut profile.destination.authentication {
                        auth.method = field.value.parse().unwrap_or(AuthenticationMethod::Password);
                    }
                }
                "destination.auth.username" => {
                    if let Some(auth) = &mut profile.destination.authentication {
                        auth.username = field.value.clone();
                    }
                }
                "destination.auth.password_ref" => {
                    if let Some(auth) = &mut profile.destination.authentication {
                        auth.password_ref = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }
                "destination.auth.private_key_ref" => {
                    if let Some(auth) = &mut profile.destination.authentication {
                        auth.private_key_ref = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }
                "destination.auth.ssh_config_alias" => {
                    if let Some(auth) = &mut profile.destination.authentication {
                        auth.ssh_config_alias = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                    }
                }

                // Transfer
                "transfer_protocol" => {
                    profile.transfer_protocol.protocol = field.value.parse().unwrap_or(ProtocolType::Sftp)
                }
                "pre_transfer_command" => {
                    profile.pre_transfer_command = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                }
                "post_transfer_command" => {
                    profile.post_transfer_command = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                }
                "on_error_command" => {
                    profile.on_error_command = if field.value.is_empty() { None } else { Some(field.value.clone()) };
                }

                _ => {}
            }
        }
    }
}

pub struct InputField {
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
    pub cursor_pos: usize,
}

impl InputField {
    pub fn new(label: impl Into<String>, value: &str, hint: Option<&str>) -> Self {
        InputField {
            label: label.into(),
            value: value.into(),
            hint: hint.map(|s| s.to_string()),
            cursor_pos: value.chars().count(),
        }
    }
}

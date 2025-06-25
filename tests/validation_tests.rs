use vento::*;

fn valid_auth() -> Authentication {
    Authentication {
        method: AuthenticationMethod::Password,
        username: "user".to_string(),
        password_ref: Some("secret".to_string()),
        private_key_ref: None,
        ssh_config_alias: None,
    }
}

#[test]
fn test_source_validate_local_success() {
    let source = Source {
        kind: SourceType::Local,
        path: "/tmp/source".into(),
        host: None,
        port: None,
        authentication: None,
        trigger: Trigger {
            kind: TriggerType::Manual,
            schedule: None,
        },
    };
    assert!(source.validate().is_ok());
}

#[test]
fn test_source_validate_sftp_missing_authentication() {
    let source = Source {
        kind: SourceType::Sftp,
        path: "/tmp/sftp".into(),
        host: Some("example.com".into()),
        port: Some(22),
        authentication: None,
        trigger: Trigger {
            kind: TriggerType::Manual,
            schedule: None,
        },
    };
    let result = source.validate();
    assert!(matches!(result, Err(AppError::Validation(msg)) if msg.contains("authentication")));
}

#[test]
fn test_destination_validate_sftp_success() {
    let destination = Destination {
        kind: DestinationType::Sftp,
        path: "/tmp/remote".into(),
        host: Some("example.com".into()),
        port: Some(22),
        authentication: Some(valid_auth()),
    };
    assert!(destination.validate().is_ok());
}

#[test]
fn test_destination_validate_sftp_missing_port() {
    let destination = Destination {
        kind: DestinationType::Sftp,
        path: "/tmp/remote".into(),
        host: Some("example.com".into()),
        port: None,
        authentication: Some(valid_auth()),
    };
    let result = destination.validate();
    assert!(matches!(result, Err(AppError::Validation(msg)) if msg.contains("port")));
}

#[test]
fn test_authentication_password_missing_password_ref() {
    let auth = Authentication {
        method: AuthenticationMethod::Password,
        username: "user".into(),
        password_ref: None,
        private_key_ref: None,
        ssh_config_alias: None,
    };
    let result = auth.validate();
    assert!(
        matches!(result, Err(AppError::AuthenticationFailed(msg)) if msg.contains("passwordRef"))
    );
}

#[test]
fn test_authentication_ssh_config_missing_alias() {
    let auth = Authentication {
        method: AuthenticationMethod::SshConfig,
        username: "user".into(),
        password_ref: None,
        private_key_ref: None,
        ssh_config_alias: None,
    };
    let result = auth.validate();
    assert!(matches!(result, Err(AppError::Validation(msg)) if msg.contains("sshConfigAlias")));
}

#[test]
fn test_trigger_manual_ok() {
    let trigger = Trigger {
        kind: TriggerType::Manual,
        schedule: None,
    };
    assert!(trigger.validate().is_ok());
}

#[test]
fn test_trigger_schedule_valid_cron() {
    let trigger = Trigger {
        kind: TriggerType::Schedule,
        schedule: Some("0 0 * * * *".into()), // 毎時0分
    };
    assert!(trigger.validate().is_ok());
}

#[test]
fn test_trigger_schedule_missing_schedule() {
    let trigger = Trigger {
        kind: TriggerType::Schedule,
        schedule: None,
    };
    let result = trigger.validate();
    assert!(matches!(result, Err(AppError::MissingSchedule)));
}

#[test]
fn test_trigger_schedule_invalid_cron() {
    let trigger = Trigger {
        kind: TriggerType::Schedule,
        schedule: Some("invalid_cron".into()),
    };
    let result = trigger.validate();
    assert!(matches!(result, Err(AppError::InvalidCronSchedule { .. })));
}

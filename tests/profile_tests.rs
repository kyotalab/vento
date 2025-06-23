
use tempfile::NamedTempFile;
use std::io::Write;

use vento::{Profile, AppError}; // 適宜クレート名を変更
use vento::SourceType;
use vento::AuthenticationMethod;
use vento::TriggerType;

#[test]
fn test_load_valid_profile_yaml() {
    let yaml = r#"
transferProfiles:
  - profileId: "test"
    description: "test description"
    source:
      type: sftp
      path: "/source"
      host: "example.com"
      port: 22
      authentication:
        method: password
        username: "user"
        passwordRef: "secret"
      trigger:
        type: manual
        schedule: null
    destination:
      type: local
      path: "/dest"
      host: null
      port: null
      authentication: null
    transferProtocol:
      protocol: SFTP
"#;

    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", yaml).unwrap();

    let path = file.path();
    let result = Profile::load_profiles(path);
    assert!(result.is_ok(), "Expected profile to load correctly");
}

#[test]
fn test_invalid_sftp_source_missing_host() {
    use vento::{Source, Authentication, Trigger};

    let source = Source {
        kind: SourceType::Sftp,
        path: "/some/path".into(),
        host: None,
        port: Some(22),
        authentication: Some(Authentication {
            method: AuthenticationMethod::Password,
            username: "user".into(),
            password_ref: Some("ref".into()),
            private_key_ref: None,
            ssh_config_alias: None,
        }),
        trigger: Trigger {
            kind: TriggerType::Manual,
            schedule: None,
        },
    };

    let result = source.validate();
    assert!(matches!(result, Err(AppError::Validation(msg)) if msg.contains("host")));
}

#[test]
fn test_trigger_schedule_missing_schedule() {
    use vento::Trigger;

    let trigger = Trigger {
        kind: TriggerType::Schedule,
        schedule: None,
    };

    let result = trigger.validate();
    assert!(matches!(result, Err(AppError::MissingSchedule)));
}

#[test]
fn test_trigger_schedule_invalid_cron() {
    use vento::Trigger;

    let trigger = Trigger {
        kind: TriggerType::Schedule,
        schedule: Some("invalid_cron".into()),
    };

    let result = trigger.validate();
    assert!(matches!(result, Err(AppError::InvalidCronSchedule { .. })));
}

#[test]
fn test_authentication_missing_private_key_ref() {
    use vento::Authentication;

    let auth = Authentication {
        method: AuthenticationMethod::PrivateKey,
        username: "user".into(),
        password_ref: None,
        private_key_ref: None,
        ssh_config_alias: None,
    };

    let result = auth.validate();
    assert!(matches!(result, Err(AppError::Validation(msg)) if msg.contains("privateKeyRef")));
}

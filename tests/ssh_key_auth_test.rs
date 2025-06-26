use std::env;
use vento::{get_private_key_path, AppError, Authentication, AuthenticationMethod};

#[test]
fn test_get_private_key_path_envkey_success() {
    unsafe {
        env::set_var("MY_KEY_PATH", "/home/user/.ssh/id_rsa");
    }
    let auth = Authentication {
        method: AuthenticationMethod::EnvKey,
        username: "user".into(),
        password_ref: None,
        private_key_ref: Some("MY_KEY_PATH".into()),
        ssh_config_alias: None,
    };
    let path = get_private_key_path(&auth).unwrap();
    assert_eq!(path, "/home/user/.ssh/id_rsa");
}

#[test]
fn test_get_private_key_path_envkey_missing_env() {
    unsafe {
        env::remove_var("MISSING_KEY");
    }
    let auth = Authentication {
        method: AuthenticationMethod::EnvKey,
        username: "user".into(),
        password_ref: None,
        private_key_ref: Some("MISSING_KEY".into()),
        ssh_config_alias: None,
    };
    let err = get_private_key_path(&auth).unwrap_err();

    if let AppError::Validation(msg) = err {
        assert!(msg.contains("MISSING_KEY"));
    } else {
        panic!("Expected Validation error with MISSING_KEY, got {:?}", err);
    }
}

#[test]
fn test_get_private_key_path_private_key_success() {
    let auth = Authentication {
        method: AuthenticationMethod::PrivateKey,
        username: "user".into(),
        password_ref: None,
        private_key_ref: Some("/path/to/key".into()),
        ssh_config_alias: None,
    };
    let path = get_private_key_path(&auth).unwrap();
    assert_eq!(path, "/path/to/key");
}

#[test]
fn test_get_private_key_path_missing_key_ref() {
    let auth = Authentication {
        method: AuthenticationMethod::PrivateKey,
        username: "user".into(),
        password_ref: None,
        private_key_ref: None,
        ssh_config_alias: None,
    };
    let err = get_private_key_path(&auth).unwrap_err();
    assert!(matches!(err, AppError::MissingPrivateKeyReference));
}

#[test]
fn test_get_private_key_path_unsupported_method() {
    let auth = Authentication {
        method: AuthenticationMethod::SshConfig,
        username: "user".into(),
        password_ref: None,
        private_key_ref: Some("dummy".into()),
        ssh_config_alias: None,
    };
    let err = get_private_key_path(&auth).unwrap_err();
    assert!(matches!(err, AppError::AuthenticationFailed(_)));
}

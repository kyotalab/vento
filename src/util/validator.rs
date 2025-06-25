use validator::ValidationError;

pub fn validate_ascii(val: &str) -> Result<(), ValidationError> {
    if val.is_ascii() {
        Ok(())
    } else {
        Err(ValidationError::new("non_ascii"))
    }
}

pub fn validate_cross_platform_path(val: &str) -> Result<(), ValidationError> {
    let forbidden = ['<', '>', ':', '"', '|', '?', '*'];
    if !val.is_ascii() {
        return Err(ValidationError::new("non_ascii_path"));
    }

    if val.chars().any(|c| forbidden.contains(&c)) {
        return Err(ValidationError::new("invalid_char_in_path"));
    }

    // 必要なら空白も弾く
    if val.contains(' ') {
        return Err(ValidationError::new("path_contains_space"));
    }

    Ok(())
}

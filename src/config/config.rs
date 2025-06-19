use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct AppConfig {
    pub default_profile_file: Option<String>,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
    pub features: Option<Features>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Features {
    // ここのプラグイン機能を追加していく感じ
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            default_profile_file: None,
            log_level: None,
            log_file: None,
            features: None,
        }
    }

    pub fn set_default_profile_file(&mut self, file: String) {
        self.default_profile_file = Some(file);
    }

    pub fn set_log_level(&mut self, level: String) {
        self.log_level = Some(level);
    }

    pub fn set_log_file(&mut self, file: String) {
        self.log_file = Some(file);
    }

    pub fn set_features(&mut self, features: Features) {
        self.features = Some(features);
    }
}

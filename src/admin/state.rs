use crate::{AppConfig, Profile};


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

pub struct InputField {
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
}

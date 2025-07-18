use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{fs, io};
use ratatui::{backend::CrosstermBackend, Terminal};
use anyhow::Result;

use crate::{render_admin, AdminMode, AdminState, AppConfig, EditState, Profile, TransferProfile, UiState};



pub fn run_admin_ui(config: AppConfig, profiles: Profile) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AdminState {
        mode: AdminMode::Profile,
        ui_state: crate::UiState::ListView,
        selected_index: 0,
        profiles,
        config,
    };


    // メインループ
    // main loop
    loop {
        terminal.draw(|f| render_admin(f, &state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // falseが返ってきたら終了
                let continue_running = handle_key_event(key_event, &mut state)?;
                if !continue_running {
                    break;
                }
            }
        }
    }

    // クリーンアップ
    // clean up
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn handle_key_event(event: KeyEvent, state: &mut AdminState) -> Result<bool> {
    let key = event.code;
    let modifiers = event.modifiers;

    match &mut state.ui_state {
        UiState::ListView => match key {
            KeyCode::Tab => {
                state.mode = match state.mode {
                    AdminMode::Profile => AdminMode::Config,
                    AdminMode::Config => AdminMode::Profile,
                };
                state.selected_index = 0;
            }
            KeyCode::Down => {
                let max = match state.mode {
                    AdminMode::Profile => state.profiles.transfer_profiles.len(),
                    AdminMode::Config => 3,
                };
                if state.selected_index + 1 < max {
                    state.selected_index += 1;
                }
            }
            KeyCode::Up => {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
            }
            KeyCode::Enter => {
                match state.mode {
                    AdminMode::Profile => {
                        if let Some(profile) = state.profiles.transfer_profiles.get(state.selected_index) {
                            state.ui_state = UiState::EditView(EditState::from_profile(profile));
                        }
                    }
                    AdminMode::Config => {
                        state.ui_state = UiState::EditView(EditState::from_config(&state.config));
                    }
                }
            }
            KeyCode::Char('n') if modifiers.contains(KeyModifiers::CONTROL) => {
                let new_profile = TransferProfile::default();
                state.profiles.transfer_profiles.push(new_profile);
                state.selected_index = state.profiles.transfer_profiles.len() - 1;
                let profile_ref = state.profiles.transfer_profiles.get(state.selected_index).unwrap();
                state.ui_state = UiState::EditView(EditState::from_profile(profile_ref));
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                if let AdminMode::Profile = state.mode {
                    if state.selected_index < state.profiles.transfer_profiles.len() {
                        state.profiles.transfer_profiles.remove(state.selected_index);
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                        }
                        let path = shellexpand::tilde(
                            state.config.default_profile_file.as_deref().unwrap_or("~/.config/vento/profiles.yaml")
                        ).to_string();
                        fs::write(&path, serde_yaml::to_string(&state.profiles)?)?;
                    }
                }
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                if let AdminMode::Profile = state.mode {
                    if let Some(profile) = state.profiles.transfer_profiles.get(state.selected_index) {
                        let mut cloned = profile.clone();
                        cloned.profile_id = format!("{}_copy", cloned.profile_id); // 適当なsuffix
                        state.profiles.transfer_profiles.push(cloned);
                        state.selected_index = state.profiles.transfer_profiles.len() - 1;
                        let profile_ref = state.profiles.transfer_profiles.get(state.selected_index).unwrap();
                        state.ui_state = UiState::EditView(EditState::from_profile(profile_ref));
                    }
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                // ListView上でのq/esc → 終了
                return Ok(false);
            }
            _ => {}
        },
        UiState::EditView(edit_state) => match key {
            KeyCode::Tab | KeyCode::Down => {
                if edit_state.current_fields + 1 < edit_state.input_fields.len() {
                    edit_state.current_fields += 1;
                }
            }
            KeyCode::BackTab | KeyCode::Up => {
                if edit_state.current_fields > 0 {
                    edit_state.current_fields -= 1;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                state.ui_state = UiState::ListView;
            }
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                match state.mode {
                    AdminMode::Profile => {
                        if let Some(profile) = state.profiles.transfer_profiles.get_mut(state.selected_index) {
                            edit_state.write_back_to_profile(profile);
                            let path = shellexpand::tilde(
                                state.config.default_profile_file.as_deref().unwrap_or("~/.config/vento/profiles.yaml")
                            ).to_string();
                            fs::write(&path, serde_yaml::to_string(&state.profiles)?)?;
                        }
                    }
                    AdminMode::Config => {
                        edit_state.write_back_to_config(&mut state.config);
                        let config_path = "~/.config/vento/config.yaml"; // Or use state.config_path if it exists
                        let config_path = shellexpand::tilde(config_path).to_string();
                        fs::write(&config_path, serde_yaml::to_string(&state.config)?)?;
                    }
                }
                state.ui_state = UiState::ListView;
            }
            KeyCode::Left => {
                if let Some(field) = edit_state.input_fields.get_mut(edit_state.current_fields) {
                    if field.cursor_pos > 0 {
                        field.cursor_pos -= 1;
                    }
                }
            }
            KeyCode::Right => {
                if let Some(field) = edit_state.input_fields.get_mut(edit_state.current_fields) {
                    if field.cursor_pos < field.value.len() {
                        field.cursor_pos += 1;
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Some(field) = edit_state.input_fields.get_mut(edit_state.current_fields) {
                    let mut chars: Vec<char> = field.value.chars().collect();
                    chars.insert(field.cursor_pos, c);
                    field.value = chars.into_iter().collect();
                    field.cursor_pos += 1;
                }
            }
            KeyCode::Backspace => {
                if let Some(field) = edit_state.input_fields.get_mut(edit_state.current_fields) {
                    if field.cursor_pos > 0 {
                        let mut chars: Vec<char> = field.value.chars().collect();
                        chars.remove(field.cursor_pos - 1);
                        field.value = chars.into_iter().collect();
                        field.cursor_pos -= 1;
                    }
                }
            }
            KeyCode::Delete => {
                if let Some(field) = edit_state.input_fields.get_mut(edit_state.current_fields) {
                    if field.cursor_pos < field.value.len() {
                        field.value.remove(field.cursor_pos);
                    }
                }
            }
            _ => {}
        },
    }
    Ok(true)
}

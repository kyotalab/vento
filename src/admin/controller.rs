use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};
use anyhow::Result;

use crate::{AdminMode, AdminState, AppConfig, EditState, InputField, Profile, UiState, render_admin};



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
    loop {
        terminal.draw(|f| render_admin(f, &state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if matches!(state.ui_state, UiState::EditView(_)) {
                            state.ui_state = UiState::ListView;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Tab => {
                        state.mode = match state.mode {
                            AdminMode::Profile => AdminMode::Config,
                            AdminMode::Config => AdminMode::Profile,
                        };
                        state.selected_index = 0;
                    }
                    KeyCode::Down => {
                        match &mut state.ui_state {
                            UiState::ListView => {
                                let max = match state.mode {
                                    AdminMode::Profile => state.profiles.transfer_profiles.len(),
                                    AdminMode::Config => 3,
                                };
                                if state.selected_index + 1 < max {
                                    state.selected_index += 1;
                                }
                            }
                            UiState::EditView(edit_state) => {
                                if edit_state.current_fields + 1 < edit_state.input_fields.len() {
                                    edit_state.current_fields += 1;
                                }
                            }
                        }
                    }
                    KeyCode::Up => {
                        match &mut state.ui_state {
                            UiState::ListView => {
                                if state.selected_index > 0 {
                                    state.selected_index -= 1;
                                }
                            }
                            UiState::EditView(edit_state) => {
                                if edit_state.current_fields > 0 {
                                    edit_state.current_fields -= 1;
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let UiState::ListView = state.ui_state {
                            match state.mode {
                                AdminMode::Profile => {
                                    if let Some(profile) = state.profiles.transfer_profiles.get(state.selected_index) {
                                        state.ui_state = UiState::EditView(EditState {
                                            input_fields: vec![
                                                InputField {
                                                    label: "Profile ID".into(),
                                                    value: profile.profile_id.clone(),
                                                    hint: Some("識別用ID".into()),
                                                },
                                                InputField {
                                                    label: "Description".into(),
                                                    value: profile.description.clone().unwrap_or_default(),
                                                    hint: Some("説明 (任意)".into()),
                                                },
                                                // 必要に応じて他フィールドも
                                            ],
                                            current_fields: 0,
                                        });
                                    }
                                }
                                AdminMode::Config => {
                                    // config編集モードが必要ならここに追加
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // クリーンアップ
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};
use anyhow::Result;

use crate::{AdminMode, AdminState, AppConfig, Profile, render_admin};



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
                    KeyCode::Char('q') | KeyCode::Esc => break,
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
                            AdminMode::Config => 3, // 表示用：Log Levelなど
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

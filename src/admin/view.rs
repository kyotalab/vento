use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use crate::{AdminMode, AdminState, SourceType};

pub fn render_admin(f: &mut Frame, state: &AdminState) {
    // レイアウトを3分割：タブ, メイン表示, ヘルプ
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // タブ
            Constraint::Min(0),     // メイン（Profile/Config）
            Constraint::Length(1),  // ヘルプ（キー操作説明）
        ])
        .split(f.area());

    // モード表示タブ
    let titles: Vec<_> = ["Profile", "Config"]
        .iter()
        .map(|t| (*t).into())
        .collect::<Vec<String>>();
    let tabs = ratatui::widgets::Tabs::new(titles)
        .select(match state.mode {
            AdminMode::Profile => 0,
            AdminMode::Config => 1,
        })
        .block(Block::default().borders(Borders::ALL).title("Mode"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    match state.mode {
        AdminMode::Profile => {
            render_profile_list(f, chunks[1], state);
        }
        AdminMode::Config => {
            render_config_summary(f, chunks[1], state);
        }
    }

    // キー操作ヘルプの表示
    let help = Paragraph::new("[↑↓] 選択  [Enter] 編集  [Q/Esc] 終了")
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[2]);
}

fn render_profile_list(f: &mut Frame, area: Rect, state: &AdminState) {
    let header = Row::new(vec![
        Cell::from("Profile ID"),
        Cell::from("Description"),
        Cell::from("Hostname"),
        Cell::from("Protocol"),
        Cell::from("Source"),
        Cell::from("Destination"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = state.profiles.transfer_profiles.iter().enumerate().map(|(i, p)| {
        let style = if i == state.selected_index {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default()
        };

        let host = if p.source.kind == SourceType::Local {
            p.destination.host.clone().unwrap_or_else(|| "<missing host>".to_string())
        } else {
            p.source.host.clone().unwrap_or_else(|| "<missing host>".to_string())
        };

        Row::new(vec![
            Cell::from(p.profile_id.clone()),
            Cell::from(p.description
                .clone()
                .unwrap_or_else(|| "".into())),
            Cell::from(host),
            Cell::from(p.transfer_protocol.protocol.to_string().clone()),
            Cell::from(p.source.path.clone()),
            Cell::from(p.destination.path.clone()),
        ])
        .style(style)
    }).collect();

    let widths = &[
            Constraint::Length(30),
            Constraint::Length(60),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Profiles").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_config_summary(f: &mut Frame, area: Rect, state: &AdminState) {
    let cfg = &state.config;

    let rows = vec![
        Row::new(vec![
            Cell::from("Default Profile Path"),
            Cell::from(cfg.default_profile_file
                .clone()
                .unwrap_or_else(|| "~/.config/vento/profiles.yaml".into())),
        ]),
        Row::new(vec![
            Cell::from("Log Level"),
            Cell::from(cfg.log_level.clone().unwrap_or_else(|| "Info".into())),
        ]),
        Row::new(vec![
            Cell::from("Log File"),
            Cell::from(cfg.log_file.clone().unwrap_or_else(|| "".into())), // 任意なので空でもOK
        ]),
        Row::new(vec![
            Cell::from("Log Stdout"),
            Cell::from(cfg
                .log_stdout
                .unwrap_or(true)
                .to_string()),
        ]),
        Row::new(vec![
            Cell::from("Max File Size(MB)"),
            Cell::from(cfg
                .max_file_size_mb
                .unwrap_or(500)
                .to_string()),
        ]),
    ];

    let widths = &[
            Constraint::Length(30),
            Constraint::Min(40),
        ];
    let table = Table::new(rows, widths)
        .block(Block::default().title("Config").borders(Borders::ALL));

    f.render_widget(table, area);
}


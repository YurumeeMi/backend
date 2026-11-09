use crate::app::state::{AppState, AuthMode};
use crate::ui::theme;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(frame: &mut Frame, app: &AppState) {
    let area = centered_rect(50, 60, frame.area());

    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(area);

    let title = match app.auth_mode {
        AuthMode::Login => "Sumi -> Login",
        AuthMode::Register => " Sumi - Account Create",
    };

    frame.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme::ACCENT)),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new("Email").style(Style::default().fg(theme::TEXT_SECONDARY)),
        chunks[1],
    );

    let email_style = if app.auth_focus == 0 {
        theme::ACCENT
    } else {
        theme::BORDER
    };

    frame.render_widget(
        Paragraph::new(format!("{}_", app.auth_email))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(email_style)),
            )
            .style(Style::default().fg(theme::TEXT_PRIMARY)),
        chunks[2],
    );

    frame.render_widget(
        Paragraph::new("password").style(Style::default().fg(theme::TEXT_SECONDARY)),
        chunks[3],
    );

    let password_style = if app.auth_focus == 1 {
        theme::ACCENT
    } else {
        theme::BORDER
    };

    frame.render_widget(
        Paragraph::new(format!("{}_", "*".repeat(app.auth_password.len())))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(password_style)),
            )
            .style(Style::default().fg(theme::TEXT_PRIMARY)),
        chunks[4],
    );

    let action = match app.auth_mode {
        AuthMode::Login => "ENTER: Login  |  R: Register",
        AuthMode::Register => "ENTER: Register  |  R: Login",
    };

    frame.render_widget(
        Paragraph::new(action)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme::TEXT_SECONDARY)),
        chunks[6],
    );

    if let Some(error) = &app.auth_error {
        frame.render_widget(
            Paragraph::new(error.as_str())
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme::DANGER)),
            chunks[7],
        );
    }

    let _ = area;
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

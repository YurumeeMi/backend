use crate::app::state::AppState;
use crate::ui::theme;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("SUMILEDGER")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme::ACCENT)),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new("Select an organization").alignment(Alignment::Center),
        chunks[1],
    );

    let items: Vec<ListItem> = app
        .organizations
        .iter()
        .map(|org| ListItem::new(format!("{} ({})", org.name, org.role)))
        .collect();

    let mut list_state = ListState::default();

    if !app.organizations.is_empty() {
        list_state.select(Some(
            app.organization_focus.min(app.organizations.len() - 1),
        ));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title("Organizations")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::ACCENT_MUTED),
        );

    frame.render_stateful_widget(list, chunks[2], &mut list_state);

    let footer = if app.organizations.is_empty() {
        "No organizations yet. Press N to create one."
    } else {
        "Up/Down: Navigate | Enter: Select | N: New organization"
    };

    frame.render_widget(
        Paragraph::new(footer).alignment(Alignment::Center),
        chunks[3],
    );

    if app.organization_create_open {
        let popup = centered_rect(50, 35, frame.area());

        frame.render_widget(Clear, popup);

        let error = app.organization_error.as_deref().unwrap_or("");

        let content = Paragraph::new(format!(
            "Organization name:\n\n{}_\n\n{}\n\nEnter: Create | Esc: Cancel",
            app.organization_name, error
        ))
        .block(
            Block::default()
                .title("New Organization")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT)),
        )
        .style(Style::default().fg(theme::TEXT_PRIMARY));

        frame.render_widget(content, popup);
    } else if let Some(error) = &app.organization_error {
        frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(theme::DANGER)),
            chunks[3],
        );
    }
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

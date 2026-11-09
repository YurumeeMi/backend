use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{app::state::AppState, ui::theme};

pub fn render(frame: &mut Frame, app: &AppState) {
    let area = centered_rect(60, 75, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" NEW ACCOUNT ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let code_style = field_style(app.account_form_focus == 0);
    let name_style = field_style(app.account_form_focus == 1);
    let type_style = field_style(app.account_form_focus == 2);
    let menu_style = field_style(app.account_form_focus == 1);
    let type_style = field_style(app.account_form_focus == 2);
    let balance_style = field_style(app.account_form_focus == 3);

    frame.render_widget(
        Paragraph::new("Account Code").style(Style::default().fg(theme::TEXT_SECONDARY)),
        sections[0],
    );

    frame.render_widget(
        Paragraph::new(format!("{}", app.account_code)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(code_style),
        ),
        sections[1],
    );

    frame.render_widget(
        Paragraph::new("Account Name").style(Style::default().fg(theme::TEXT_SECONDARY)),
        sections[2],
    );

    frame.render_widget(
        Paragraph::new(format!("{}", app.account_name)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(name_style),
        ),
        sections[3],
    );

    frame.render_widget(
        Paragraph::new("Opening Balance (₹)").style(Style::default().fg(theme::TEXT_SECONDARY)),
        sections[6],
    );

    frame.render_widget(
        Paragraph::new(format!(" ₹ {}_", app.opening_balance))
            .style(Style::default().fg(theme::TEXT_PRIMARY))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(balance_style),
            ),
        sections[7],
    );

    let account_types = ["Assets", "Liablitiy", "Equity", "Revenue", "Expense"];
    let selected_type = account_types.get(app.account_type).unwrap_or(&"Asset");

    frame.render_widget(
        Paragraph::new("Account Type").style(Style::default().fg(theme::TEXT_SECONDARY)),
        sections[4],
    );

    frame.render_widget(
        Paragraph::new(format!(" {}  (← / →)", selected_type)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(type_style),
        ),
        sections[5],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            "Tab".into(),
            " Next field    ".into(),
            "← / →".into(),
            " Change type    ".into(),
            "Enter".into(),
            " Create    ".into(),
            "Esc".into(),
            " Cancel".into(),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme::TEXT_SECONDARY)),
        sections[9],
    );
}

fn field_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::BORDER)
    }
}
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Percentage(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Percentage(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(vertical[1])[1]
}

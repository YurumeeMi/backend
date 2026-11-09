use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "Overview",
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " / Financial summary",
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
    ]));

    frame.render_widget(title, sections[0]);

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(sections[1]);

    render_card(frame, cards[0], "TOTAL ASSETS", "₹0.00", theme::SUCCESS);
    render_card(frame, cards[1], "LIABILiTIES", "₹0.00", theme::WARNING);
    render_card(frame, cards[2], "EQUITY", "₹0.00", theme::INFO);

    let activity = Paragraph::new("no Transactions recorded yet.")
        .style(Style::default().fg(theme::TEXT_SECONDARY))
        .block(
            Block::default()
                .title(" Recent Activity")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER)),
        );

    frame.render_widget(activity, sections[2]);
}

fn render_card(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    color: ratatui::style::Color,
) {
    let content = Paragraph::new(vec![
        Line::from(Span::styled(
            label,
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(""),
        Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER)),
    );
    frame.render_widget(content, area);
}

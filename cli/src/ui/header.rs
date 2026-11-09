use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            "Sumi",
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " LEDGER",
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "| Finance management",
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
    ]);

    let header = Paragraph::new(title).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER)),
    );
    frame.render_widget(header, area);
}

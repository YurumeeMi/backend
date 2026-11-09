use super::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
};

const ITEMS: [&str; 4] = [" Overview", " Accounts", " Transactions", " Reports"];

pub fn render(frame: &mut Frame, area: Rect, selected: usize) {
    let items: Vec<ListItem> = ITEMS.iter().map(|item| ListItem::new(*item)).collect();

    let mut state = ListState::default();
    state.select(Some(selected));

    let sidebar = List::new(items)
        .block(
            Block::default()
                .title(" Workspace")
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(theme::BORDER)),
        )
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::ACCENT_MUTED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("|");

    frame.render_stateful_widget(sidebar, area, &mut state);
}

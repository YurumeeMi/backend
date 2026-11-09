use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub sidebar: Rect,
    pub content: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Min(0)])
            .split(vertical[1]);

        Self {
            header: vertical[0],
            sidebar: horizontal[0],
            content: horizontal[1],
        }
    }
}

pub mod header;
pub mod layout;
pub mod pages;
pub mod sidebar;
pub mod theme;

use crate::app::state::AppState;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &AppState) {
    let layout = layout::AppLayout::new(frame.area());

    header::render(frame, layout.header);

    sidebar::render(frame, layout.sidebar, app.selected_index());

    pages::overview::render(frame, layout.content);
}

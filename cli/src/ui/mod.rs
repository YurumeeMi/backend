pub mod header;
pub mod layout;
pub mod pages;
pub mod sidebar;
pub mod theme;

use crate::app::state::AppState;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &AppState) {
    let layout = layout::AppLayout::new(frame.area());

    if app.user_id.is_none() {
        pages::login::render(frame, app);
        return;
    }
    if app.selected_organization.is_none() {
        pages::organization_selector::render(frame, app);
        return;
    }
    header::render(frame, layout.header);

    sidebar::render(frame, layout.sidebar, app.selected_index());

    match app.selected_page {
        crate::app::state::Page::Overview => {
            pages::overview::render(frame, layout.content);
        }
        crate::app::state::Page::Accounts => {
            pages::accounts::render(frame, layout.content, &app.accounts);
            if app.account_form_open {
                pages::account_form::render(frame, app);
            }
        }
        crate::app::state::Page::Transactions | crate::app::state::Page::Reports => {
            let message = ratatui::widgets::Paragraph::new("Coming soon...")
                .style(ratatui::style::Style::default().fg(theme::TEXT_SECONDARY));

            frame.render_widget(message, layout.content);
        }
    }
}

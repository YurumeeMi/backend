#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Accounts,
    Transactions,
    Reports,
}

pub struct AppState {
    pub selected_page: Page,
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_page: Page::Overview,
            should_quit: false,
        }
    }
    pub fn next_page(&mut self) {
        self.selected_page = match self.selected_page {
            Page::Overview => Page::Accounts,
            Page::Accounts => Page::Transactions,
            Page::Transactions => Page::Reports,
            Page::Reports => Page::Overview,
        };
    }

    pub fn pervious_page(&mut self) {
        self.selected_page = match self.selected_page {
            Page::Overview => Page::Accounts,
            Page::Accounts => Page::Transactions,
            Page::Transactions => Page::Reports,
            Page::Reports => Page::Overview,
        };
    }
    pub fn selected_index(&self) -> usize {
        match self.selected_page {
            Page::Overview => 0,
            Page::Accounts => 1,
            Page::Transactions => 2,
            Page::Reports => 3,
        }
    }
}

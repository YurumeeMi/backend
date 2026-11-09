use super::account::Account;
use crate::api::organizations::Organization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Accounts,
    Transactions,
    Reports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Login,
    Register,
}

pub struct AppState {
    pub selected_page: Page,
    pub should_quit: bool,
    pub account_form_open: bool,
    pub account_error: Option<String>,
    pub account_code: String,
    pub account_name: String,
    pub account_type: usize,
    pub account_form_focus: usize,
    pub opening_balance: String,
    pub accounts: Vec<Account>,
    pub auth_mode: AuthMode,
    pub auth_email: String,
    pub auth_password: String,
    pub auth_focus: usize,
    pub auth_error: Option<String>,
    pub is_loading: bool,
    pub user_id: Option<String>,
    pub organizations: Vec<Organization>,
    pub selected_organization: Option<String>,
    pub organization_focus: usize,
    pub organization_name: String,
    pub organization_error: Option<String>,
    pub organization_create_open: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_page: Page::Overview,
            should_quit: false,
            account_form_open: false,
            account_error: None,
            account_code: String::new(),
            account_name: String::new(),
            account_type: 0,
            account_form_focus: 0,
            opening_balance: String::from("0.00"),
            accounts: vec![],
            auth_mode: AuthMode::Login,
            auth_email: String::new(),
            auth_password: String::new(),
            auth_focus: 0,
            auth_error: None,
            is_loading: false,
            user_id: None,
            organizations: Vec::new(),
            selected_organization: None,
            organization_focus: 0,
            organization_name: String::new(),
            organization_error: None,
            organization_create_open: false,
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
    pub fn open_account_form(&mut self) {
        self.account_form_open = true;
        self.account_form_focus = 0;
        self.account_code.clear();
        self.account_name.clear();
        self.account_type = 0;
        self.opening_balance = String::from("0.00")
    }

    pub fn close_account_form(&mut self) {
        self.account_form_open = false;
    }

    pub fn next_form_field(&mut self) {
        self.account_form_focus = (self.account_form_focus + 1) % 4;
    }

    pub fn previous_form_field(&mut self) {
        self.account_form_focus = (self.account_form_focus + 3) % 4;
    }
}

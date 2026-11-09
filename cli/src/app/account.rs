#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub opening_balance: String,
    pub is_active: bool,
}

impl From<crate::api::accounts::Account> for Account {
    fn from(account: crate::api::accounts::Account) -> Self {
        Self {
            id: account.id,
            code: account.code,
            name: account.name,
            account_type: account.account_type,
            opening_balance: account.opening_balance,
            is_active: account.is_active,
        }
    }
}

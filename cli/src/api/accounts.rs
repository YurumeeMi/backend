use super::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub opening_balance: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct AccountListResponse {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub opening_balance: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountResponse {
    pub id: String,
}

impl ApiClient {
    pub async fn get_accounts(
        &self,
        organization_id: &str,
    ) -> Result<AccountListResponse, reqwest::Error> {
        self.http
            .get(format!(
                "{}/organizations/{}/accounts",
                self.base_url, organization_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn create_account(
        &self,
        organization_id: &str,
        code: &str,
        name: &str,
        account_type: &str,
        opening_balance: &str,
    ) -> Result<CreateAccountResponse, reqwest::Error> {
        let request = CreateAccountRequest {
            code: code.to_string(),
            name: name.to_string(),
            account_type: account_type.to_string(),
            opening_balance: opening_balance.to_string(),
        };

        self.http
            .post(format!(
                "{}/organizations/{}/accounts",
                self.base_url, organization_id
            ))
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

use super::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub id: String,
}

impl ApiClient {
    pub async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, reqwest::Error> {
        self.http
            .post(format!("{}/auth/register", self.base_url))
            .json(&RegisterRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<AuthResponse>()
            .await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, reqwest::Error> {
        self.http
            .post(format!("{}/auth/login", self.base_url))
            .json(&LoginRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<AuthResponse>()
            .await
    }

    pub async fn current_user(&self) -> Result<AuthResponse, reqwest::Error> {
        self.http
            .get(format!("{}/auth/me", self.base_url))
            .send()
            .await?
            .error_for_status()?
            .json::<AuthResponse>()
            .await
    }

    pub async fn logout(&self) -> Result<(), reqwest::Error> {
        self.http
            .post(format!("{}/auth/logout", self.base_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

use super::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationListResponse {
    pub organizations: Vec<Organization>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationResponse {
    pub id: String,
}

impl ApiClient {
    pub async fn get_organizations(&self) -> Result<OrganizationListResponse, reqwest::Error> {
        self.http
            .get(format!("{}/organizations", self.base_url))
            .send()
            .await?
            .error_for_status()?
            .json::<OrganizationListResponse>()
            .await
    }

    pub async fn create_organization(
        &self,
        name: &str,
    ) -> Result<CreateOrganizationResponse, reqwest::Error> {
        self.http
            .post(format!("{}/organizations", self.base_url))
            .json(&CreateOrganizationRequest {
                name: name.to_string(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<CreateOrganizationResponse>()
            .await
    }
}

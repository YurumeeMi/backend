use reqwest::Client;

pub struct ApiClient {
    pub http: Client,
    pub base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, reqwest::Error> {
        let http = Client::builder().cookie_store(true).build()?;

        Ok(Self {
            http,
            base_url: base_url.into(),
        })
    }
}

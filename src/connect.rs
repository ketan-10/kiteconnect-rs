use crate::constants::{Endpoints, app_constants::*};
use reqwest::Client;
use web_time::Duration;

pub struct KiteConnect {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) http_client: Client,
    pub(crate) access_token: Option<String>,
}

impl KiteConnect {
    pub fn builder(api_key: &str) -> KiteConnectBuilder {
        KiteConnectBuilder::new(api_key)
    }

    pub fn get_login_url(&self) -> String {
        format!(
            "{}{} ?api_key={}&v={}",
            KITE_BASE_URL,
            Endpoints::LOGIN_URL,
            self.api_key,
            KITE_HEADER_VERSION
        )
    }

    pub fn set_access_token(&mut self, token: &str) {
        self.access_token = Some(token.to_owned());
    }

    pub fn clear_access_token(&mut self) {
        self.access_token = None;
    }

    /// Get the current access token (for testing purposes)
    #[cfg(test)]
    pub fn get_access_token(&self) -> Option<&String> {
        self.access_token.as_ref()
    }

    /// Get the API key (for testing purposes)
    #[cfg(test)]
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

pub struct KiteConnectBuilder {
    api_key: String,
    access_token: Option<String>,
    base_url: Option<String>,
    http_client: Option<Client>,
    timeout: Option<Duration>,
}

impl KiteConnectBuilder {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            access_token: None,
            base_url: None,
            http_client: None,
            timeout: None,
        }
    }

    pub fn access_token(mut self, token: &str) -> Self {
        self.access_token = Some(token.to_owned());
        self
    }

    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_owned());
        self
    }

    pub fn http_client(mut self, client: Client) -> Self {
        self.http_client = Some(client);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<KiteConnect, reqwest::Error> {
        let http_client = match self.http_client {
            None => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);
                    Client::builder().timeout(timeout).build()?
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // WASM doesn't support timeout on reqwest
                    Client::builder().build()?
                }
            }
            Some(client) => client,
        };
        Ok(KiteConnect {
            api_key: self.api_key,
            access_token: self.access_token,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            http_client,
        })
    }
}

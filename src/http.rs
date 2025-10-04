use reqwest::{
    Method, Response,
    header::{HeaderMap, HeaderValue},
};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Error},
};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    KiteConnectErrorKind::SerializationError,
    constants::app_constants::*,
    models::{KiteConnectError, KiteError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    data: T,
}

pub enum RequestBody<T: Serialize> {
    Form(T),
    Json(T),
}

impl KiteConnect {
    /// Central method for making authenticated API requests
    async fn do_envelope<T, K: Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: Option<HashMap<String, String>>,
        body: Option<RequestBody<K>>,
        headers: Option<HeaderMap>,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request_headers = self.get_default_headers()?;

        // Add Authorization header if access token is available
        if let Some(ref token) = self.access_token {
            request_headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("token {}:{}", self.api_key, token))?,
            );
        }

        // Merge custom headers if provided
        if let Some(custom_headers) = headers {
            for (key, value) in custom_headers.iter() {
                request_headers.insert(key, value.clone());
            }
        }

        let mut request_builder = self
            .http_client
            .request(method, &url)
            .headers(request_headers);

        // Handle query parameters if present
        if let Some(query) = query_params {
            request_builder = request_builder.query(&query);
        }

        // Handle request body if present
        if let Some(body) = body {
            match body {
                RequestBody::Form(form_params) => {
                    request_builder = request_builder.form(&form_params);
                }
                RequestBody::Json(json_body) => {
                    request_builder = request_builder.json(&json_body);
                }
            }
        }

        let response = request_builder.send().await?;
        self.handle_response(response).await
    }

    /// Handle the response and parse it into the expected type
    async fn handle_response<T>(&self, response: Response) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            // Try to parse as wrapped response first
            if let Ok(api_response) = serde_json::from_str::<ApiResponse<T>>(&response_text) {
                Ok(api_response.data)
            } else if let Ok(result) = serde_json::from_str::<T>(&response_text) {
                Ok(result)
            } else if let Ok(result) =
                serde_json::from_value(serde_json::Value::String(response_text.clone()))
            {
                // if T = String or similar, return the raw text
                Ok(result)
            } else {
                // failed trying to parse as T. (type mismatch with T and response)
                let type_name = std::any::type_name::<T>();

                Err(KiteConnectError::new(SerializationError(Error::custom(
                    format!(
                        "Failed to parse response as {}. Response (first 500 chars): {}",
                        type_name,
                        &response_text.chars().take(500).collect::<String>()
                    ),
                ))))
            }
        } else {
            // Parse error response
            let error: KiteError = serde_json::from_str(&response_text)?;
            Err(error.into())
        }
    }

    /// Get default headers for all requests
    fn get_default_headers(&self) -> Result<HeaderMap, KiteConnectError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Kite-Version",
            HeaderValue::from_static(KITE_HEADER_VERSION),
        );

        let user_agent = HeaderValue::from_str(&format!(
            "{}/{}",
            KITE_CONNECT_RS_NAME, KITE_CONNECT_RS_VERSION
        ))?;
        headers.insert("User-Agent", user_agent);

        Ok(headers)
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::GET, endpoint, None, None, None)
            .await
    }

    pub async fn put<T>(&self, endpoint: &str) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::PUT, endpoint, None, None, None)
            .await
    }
    pub async fn post<T>(&self, endpoint: &str) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::POST, endpoint, None, None, None)
            .await
    }
    pub async fn delete<T>(&self, endpoint: &str) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::DELETE, endpoint, None, None, None)
            .await
    }

    /// Make a POST request with form parameters
    pub async fn post_form<T, K: Serialize>(
        &self,
        endpoint: &str,
        params: K,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope(
            Method::POST,
            endpoint,
            None,
            Some(RequestBody::Form(params)),
            None,
        )
        .await
    }

    /// Make a POST request with JSON body
    pub async fn post_json<T, K: Serialize>(
        &self,
        endpoint: &str,
        json_body: K,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope(
            Method::POST,
            endpoint,
            None,
            Some(RequestBody::Json(json_body)),
            None,
        )
        .await
    }

    /// Make a DELETE request with form parameters
    pub async fn delete_form<T, K: Serialize>(
        &self,
        endpoint: &str,
        params: K,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope(
            Method::DELETE,
            endpoint,
            None,
            Some(RequestBody::Form(params)),
            None,
        )
        .await
    }

    /// Make a PUT request with form parameters
    pub async fn put_form<T, K: Serialize>(
        &self,
        endpoint: &str,
        params: K,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope(
            Method::PUT,
            endpoint,
            None,
            Some(RequestBody::Form(params)),
            None,
        )
        .await
    }

    /// Make a PUT request with JSON body
    pub async fn put_json<T, K: Serialize>(
        &self,
        endpoint: &str,
        json_body: K,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope(
            Method::PUT,
            endpoint,
            None,
            Some(RequestBody::Json(json_body)),
            None,
        )
        .await
    }

    /// Make a GET request with query parameters
    pub async fn get_with_query<T>(
        &self,
        endpoint: &str,
        params: HashMap<String, String>,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::GET, endpoint, Some(params), None, None)
            .await
    }

    /// Make a DELETE request with query parameters
    pub async fn delete_with_query<T>(
        &self,
        endpoint: &str,
        params: HashMap<String, String>,
    ) -> Result<T, KiteConnectError>
    where
        T: DeserializeOwned,
    {
        self.do_envelope::<T, ()>(Method::DELETE, endpoint, Some(params), None, None)
            .await
    }
}

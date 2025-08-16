use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    constants::Endpoints,
    models::{KiteConnectError, time},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub user_name: String,
    pub user_shortname: String,
    pub avatar_url: Option<String>,
    pub user_type: String,
    pub email: String,
    pub broker: String,
    pub meta: UserMeta,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub exchanges: Vec<String>,

    // Session tokens
    pub access_token: String,
    pub refresh_token: String,

    pub api_key: String,
    pub public_token: String,
    pub login_time: time::Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionTokens {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    pub name: String,
    pub branch: String,
    pub account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMeta {
    pub demat_consent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullUserMeta {
    #[serde(rename = "poa")]
    pub demat_consent: String,
    pub silo: String,
    pub account_blocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub user_name: String,
    pub user_shortname: String,
    pub avatar_url: Option<String>,
    pub user_type: String,
    pub email: String,
    pub broker: String,
    pub meta: UserMeta,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub exchanges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullUserProfile {
    pub user_id: String,
    pub user_name: String,
    pub avatar_url: Option<String>,
    pub user_type: String,
    pub email: String,
    pub phone: String,
    pub broker: String,
    pub twofa_type: String,
    #[serde(rename = "bank_accounts")]
    pub banks: Vec<Bank>,
    pub dp_ids: Vec<String>,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub exchanges: Vec<String>,
    pub pan: String,
    pub user_shortname: String,
    pub tags: Vec<String>,
    pub password_timestamp: String,
    pub twofa_timestamp: String,
    pub meta: FullUserMeta,
}

// Margins represents the user margins for a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    #[serde(skip)] // Equivalent to `json:"-"`
    pub category: String,
    pub enabled: bool,
    pub net: f64,
    pub available: AvailableMargins,
    #[serde(rename = "utilised")]
    pub used: UsedMargins,
}

// AvailableMargins represents the available margins from the margins response for a single segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableMargins {
    pub adhoc_margin: f64,
    pub cash: f64,
    pub collateral: f64,
    pub intraday_payin: f64,
    pub live_balance: f64,
    pub opening_balance: f64,
}

// UsedMargins represents the used margins from the margins response for a single segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedMargins {
    pub debits: f64,
    pub exposure: f64,
    pub m2m_realised: f64,
    pub m2m_unrealised: f64,
    pub option_premium: f64,
    pub payout: f64,
    pub span: f64,
    pub holding_sales: f64,
    pub turnover: f64,
    pub liquid_collateral: f64,
    pub stock_collateral: f64,
    pub delivery: f64,
}

// AllMargins contains both equity and commodity margins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMargins {
    pub equity: Margins,
    pub commodity: Margins,
}

impl KiteConnect {
    /// Generate session and get user details in exchange for request token.
    /// Access token is automatically set if the session is retrieved successfully.
    pub async fn generate_session(
        &mut self,
        request_token: &str,
        api_secret: &str,
    ) -> Result<UserSession, KiteConnectError> {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", self.api_key, request_token, api_secret));
        let checksum = format!("{:x}", hasher.finalize());

        let mut params = HashMap::new();
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("request_token".to_string(), request_token.to_string());
        params.insert("checksum".to_string(), checksum);

        let session: UserSession = self.post_form(Endpoints::SESSION_GENERATE, params).await?;

        // Automatically set access token on successful session retrieve
        self.set_access_token(&session.access_token);

        Ok(session)
    }

    /// Invalidate a token (access_token or refresh_token)
    async fn invalidate_token(
        &self,
        token_type: &str,
        token: &str,
    ) -> Result<bool, KiteConnectError> {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert(token_type.to_string(), token.to_string());

        // For invalidate, we expect an empty response, so we'll handle it differently
        match self
            .delete_form::<serde_json::Value, _>(Endpoints::INVALIDATE_TOKEN, params)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Invalidate the current access token
    pub async fn invalidate_access_token(&mut self) -> Result<bool, KiteConnectError> {
        match self.access_token.clone() {
            Some(token) => {
                let result = self.invalidate_token("access_token", &token).await?;
                if result {
                    self.clear_access_token();
                }
                Ok(result)
            }
            None => Ok(false),
        }
    }

    /// Renew expired access token using valid refresh token
    /// Access token is automatically set if the renewal is successful.
    pub async fn renew_access_token(
        &mut self,
        refresh_token: &str,
        api_secret: &str,
    ) -> Result<UserSessionTokens, KiteConnectError> {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", self.api_key, refresh_token, api_secret));
        let checksum = format!("{:x}", hasher.finalize());

        let mut params = HashMap::new();
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("refresh_token".to_string(), refresh_token.to_string());
        params.insert("checksum".to_string(), checksum);

        let tokens: UserSessionTokens = self.post_form(Endpoints::RENEW_ACCESS, params).await?;

        // Automatically set access token on successful renewal
        self.set_access_token(&tokens.access_token);

        Ok(tokens)
    }

    /// Invalidate the given refresh token
    pub async fn invalidate_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<bool, KiteConnectError> {
        self.invalidate_token("refresh_token", refresh_token).await
    }

    /// Get user profile
    pub async fn get_user_profile(&self) -> Result<UserProfile, KiteConnectError> {
        self.get(Endpoints::USER_PROFILE).await
    }

    /// Get full user profile
    pub async fn get_full_user_profile(&self) -> Result<FullUserProfile, KiteConnectError> {
        self.get(Endpoints::USER_FULL_PROFILE).await
    }

    /// Get all user margins
    pub async fn get_user_margins(&self) -> Result<AllMargins, KiteConnectError> {
        self.get(Endpoints::USER_MARGINS).await
    }

    /// Get segment wise user margins
    pub async fn get_user_segment_margins(
        &self,
        segment: &str,
    ) -> Result<Margins, KiteConnectError> {
        let endpoint = Endpoints::USER_MARGINS_SEGMENT.replace("{segment}", segment);
        self.get(&endpoint).await
    }
}

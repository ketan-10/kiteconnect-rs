use crate::models::time::Time;
use crate::{KiteConnect, KiteConnectError, constants::Endpoints, models::OHLC};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    Simple,
    Ato,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Enabled,
    Disabled,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertOperator {
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">=")]
    Ge,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = "==")]
    Eq,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub r#type: AlertType,
    pub user_id: String,
    pub uuid: String,
    pub name: String,
    pub status: AlertStatus,
    pub disabled_reason: String,
    pub lhs_attribute: String,
    pub lhs_exchange: String,
    pub lhs_tradingsymbol: String,
    pub operator: AlertOperator,
    pub rhs_type: String,
    pub rhs_attribute: String,
    pub rhs_exchange: String,
    pub rhs_tradingsymbol: String,
    pub rhs_constant: Option<f64>,
    pub alert_count: Option<i32>,
    pub created_at: Option<Time>,
    pub updated_at: Option<Time>,
    pub basket: Option<Basket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertParams {
    pub name: String,
    pub r#type: AlertType,
    pub lhs_exchange: String,
    pub lhs_tradingsymbol: String,
    pub lhs_attribute: String,
    pub operator: AlertOperator,
    pub rhs_type: String,
    pub rhs_constant: Option<f64>,
    pub rhs_exchange: Option<String>,
    pub rhs_tradingsymbol: Option<String>,
    pub rhs_attribute: Option<String>,
    pub basket: Option<Basket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Basket {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub items: Vec<BasketItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasketItem {
    #[serde(default)]
    pub r#type: String,
    pub tradingsymbol: String,
    pub exchange: String,
    pub weight: i32,
    pub params: AlertOrderParams,
    pub id: Option<i32>,
    pub instrument_token: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertOrderParams {
    pub transaction_type: String,
    pub product: String,
    pub order_type: String,
    pub validity: String,
    #[serde(default)]
    pub validity_ttl: Option<i32>,
    pub quantity: i32,
    pub price: f64,
    pub trigger_price: f64,
    #[serde(default)]
    pub disclosed_quantity: Option<i32>,
    #[serde(default)]
    pub last_price: Option<f64>,
    pub variety: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub squareoff: Option<f64>,
    #[serde(default)]
    pub stoploss: Option<f64>,
    #[serde(default)]
    pub trailing_stoploss: Option<f64>,
    #[serde(default)]
    pub iceberg_legs: Option<i32>,
    #[serde(default)]
    pub market_protection: Option<f64>,
    pub gtt: Option<OrderGTTParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderGTTParams {
    pub target: f64,
    pub stoploss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertHistory {
    pub uuid: String,
    pub r#type: AlertType,
    pub meta: Vec<AlertHistoryMeta>,
    pub condition: String,
    pub created_at: Option<Time>,
    pub order_meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertHistoryMeta {
    pub instrument_token: i32,
    pub tradingsymbol: String,
    pub timestamp: String,
    pub last_price: f64,
    pub ohlc: OHLC,
    pub net_change: f64,
    pub exchange: String,
    pub last_trade_time: String,
    pub last_quantity: i32,
    pub buy_quantity: i32,
    pub sell_quantity: i32,
    pub volume: i32,
    pub volume_tick: i32,
    pub average_price: f64,
    pub oi: i32,
    pub oi_day_high: i32,
    pub oi_day_low: i32,
    pub lower_circuit_limit: f64,
    pub upper_circuit_limit: f64,
}

impl KiteConnect {
    pub async fn create_alert(&self, params: AlertParams) -> Result<Alert, KiteConnectError> {
        self.post_form(Endpoints::ALERTS_URL, &params).await
    }

    pub async fn get_alerts(
        &self,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<Alert>, KiteConnectError> {
        match filters {
            Some(f) if !f.is_empty() => self.get_with_query(Endpoints::ALERTS_URL, f).await,
            _ => self.get(Endpoints::ALERTS_URL).await,
        }
    }

    pub async fn get_alert(&self, uuid: &str) -> Result<Alert, KiteConnectError> {
        self.get(&Endpoints::ALERT_URL.replace("{alert_id}", uuid))
            .await
    }

    pub async fn modify_alert(
        &self,
        uuid: &str,
        params: AlertParams,
    ) -> Result<Alert, KiteConnectError> {
        self.put_form(&Endpoints::ALERT_URL.replace("{alert_id}", uuid), &params)
            .await
    }

    pub async fn delete_alerts(&self, uuids: &[&str]) -> Result<(), KiteConnectError> {
        if uuids.is_empty() {
            return Err(KiteConnectError::other(
                "At least one uuid must be provided",
            ));
        }

        let params = uuids
            .iter()
            .map(|&uuid| ("uuid".to_string(), uuid.to_string()))
            .collect();

        self.delete_with_query(Endpoints::ALERTS_URL, params).await
    }

    pub async fn get_alert_history(
        &self,
        uuid: &str,
    ) -> Result<Vec<AlertHistory>, KiteConnectError> {
        self.get(&Endpoints::GET_ALERT_HISTORY.replace("{alert_id}", uuid))
            .await
    }
}

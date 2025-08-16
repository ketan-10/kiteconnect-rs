use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    constants::Endpoints,
    models::{KiteConnectError, time},
};

/// Order represents an individual order response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    pub placed_by: String,

    pub order_id: String,
    pub exchange_order_id: Option<String>,
    pub parent_order_id: Option<String>,
    pub status: String,
    pub status_message: Option<String>,
    pub status_message_raw: Option<String>,
    #[serde(default)]
    pub order_timestamp: time::Time,
    #[serde(default)]
    pub exchange_update_timestamp: time::Time,
    #[serde(default)]
    pub exchange_timestamp: time::Time,
    pub variety: String,
    #[serde(default)]
    pub modified: bool,
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,

    pub exchange: String,
    pub tradingsymbol: String,
    pub instrument_token: u32,

    pub order_type: String,
    pub transaction_type: String,
    pub validity: String,
    pub validity_ttl: Option<i32>,
    pub product: String,
    pub quantity: f64,
    pub disclosed_quantity: f64,
    pub price: f64,
    pub trigger_price: f64,

    pub average_price: f64,
    pub filled_quantity: f64,
    pub pending_quantity: f64,
    pub cancelled_quantity: f64,

    pub auction_number: Option<String>,

    pub tag: Option<String>,
    pub tags: Option<Vec<String>>,

    // Additional fields that might be present in responses
    pub market_protection: Option<f64>,
    pub guid: Option<String>,
}

/// Orders is a list of orders.
pub type Orders = Vec<Order>;

/// OrderParams represents parameters for placing an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderParams {
    pub exchange: Option<String>,
    pub tradingsymbol: Option<String>,
    pub validity: Option<String>,
    pub validity_ttl: Option<i32>,
    pub product: Option<String>,
    pub order_type: Option<String>,
    pub transaction_type: Option<String>,

    pub quantity: Option<i32>,
    pub disclosed_quantity: Option<i32>,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,

    pub squareoff: Option<f64>,
    pub stoploss: Option<f64>,
    pub trailing_stoploss: Option<f64>,

    pub iceberg_legs: Option<i32>,
    pub iceberg_quantity: Option<i32>,

    pub auction_number: Option<String>,

    pub tag: Option<String>,
}

/// OrderResponse represents the order place success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
}

/// Trade represents an individual trade response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub average_price: f64,
    pub quantity: f64,
    pub trade_id: String,
    pub product: String,
    #[serde(default)]
    pub fill_timestamp: time::Time,
    #[serde(default)]
    pub exchange_timestamp: time::Time,
    pub exchange_order_id: String,
    pub order_id: String,
    pub transaction_type: String,
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,

    // Additional field that might be present
    pub order_timestamp: Option<String>,
}

/// Trades is a list of trades.
pub type Trades = Vec<Trade>;

impl KiteConnect {
    /// Gets list of orders.
    pub async fn get_orders(&self) -> Result<Orders, KiteConnectError> {
        self.get(Endpoints::GET_ORDERS).await
    }

    /// Gets list of trades.
    pub async fn get_trades(&self) -> Result<Trades, KiteConnectError> {
        self.get(Endpoints::GET_TRADES).await
    }

    /// Gets history of an individual order.
    pub async fn get_order_history(&self, order_id: &str) -> Result<Vec<Order>, KiteConnectError> {
        let endpoint = &Endpoints::GET_ORDER_HISTORY.replace("{order_id}", order_id);
        self.get(endpoint).await
    }

    /// Gets list of trades executed for a particular order.
    pub async fn get_order_trades(&self, order_id: &str) -> Result<Vec<Trade>, KiteConnectError> {
        let endpoint = &Endpoints::GET_ORDER_TRADES.replace("{order_id}", order_id);
        self.get(endpoint).await
    }

    /// Places an order.
    pub async fn place_order(
        &self,
        variety: &str,
        order_params: OrderParams,
    ) -> Result<OrderResponse, KiteConnectError> {
        let endpoint = &Endpoints::PLACE_ORDER.replace("{variety}", variety);
        println!("{:?} ", order_params);
        self.post_form(endpoint, order_params).await
    }

    /// Modifies an order.
    pub async fn modify_order(
        &self,
        variety: &str,
        order_id: &str,
        order_params: OrderParams,
    ) -> Result<OrderResponse, KiteConnectError> {
        let endpoint = &Endpoints::MODIFY_ORDER
            .replace("{variety}", variety)
            .replace("{order_id}", order_id);
        println!("{:?} ", order_params);
        self.put_form(endpoint, order_params).await
    }

    /// Cancels/exits an order.
    pub async fn cancel_order(
        &self,
        variety: &str,
        order_id: &str,
        parent_order_id: Option<&str>,
    ) -> Result<OrderResponse, KiteConnectError> {
        let endpoint = &Endpoints::CANCEL_ORDER
            .replace("{variety}", variety)
            .replace("{order_id}", order_id);

        let mut params = HashMap::new();
        if let Some(parent_id) = parent_order_id {
            params.insert("parent_order_id".to_string(), parent_id.to_string());
        }

        self.delete_form(endpoint, params).await
    }

    /// Alias for cancel_order which is used to cancel/exit an order.
    pub async fn exit_order(
        &self,
        variety: &str,
        order_id: &str,
        parent_order_id: Option<&str>,
    ) -> Result<OrderResponse, KiteConnectError> {
        self.cancel_order(variety, order_id, parent_order_id).await
    }
}

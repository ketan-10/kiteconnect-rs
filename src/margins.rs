use serde::{Deserialize, Serialize};

use crate::{KiteConnect, constants::Endpoints, models::KiteConnectError};

/// OrderMarginParam represents an order in the Margin Calculator API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMarginParam {
    pub exchange: String,
    #[serde(rename = "tradingsymbol")]
    pub trading_symbol: String,
    pub transaction_type: String,
    pub variety: String,
    pub product: String,
    pub order_type: String,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
}

/// OrderChargesParam represents an order in the Charges Calculator API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChargesParam {
    pub order_id: String,
    pub exchange: String,
    #[serde(rename = "tradingsymbol")]
    pub trading_symbol: String,
    pub transaction_type: String,
    pub variety: String,
    pub product: String,
    pub order_type: String,
    pub quantity: f64,
    pub average_price: f64,
}

/// PNL represents the PNL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNL {
    pub realised: f64,
    pub unrealised: f64,
}

/// GST represents the various GST charges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GST {
    pub igst: f64,
    pub cgst: f64,
    pub sgst: f64,
    pub total: f64,
}

/// Charges represents breakdown of various charges that are applied to an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charges {
    pub transaction_tax: f64,
    pub transaction_tax_type: String,
    pub exchange_turnover_charge: f64,
    pub sebi_turnover_charge: f64,
    pub brokerage: f64,
    pub stamp_duty: f64,
    pub gst: GST,
    pub total: f64,
}

/// OrderMargins represents response from the Margin Calculator API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMargins {
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "tradingsymbol")]
    pub trading_symbol: String,
    pub exchange: String,

    #[serde(default)]
    pub span: f64,
    #[serde(default)]
    pub exposure: f64,
    #[serde(default)]
    pub option_premium: f64,
    #[serde(default)]
    pub additional: f64,
    #[serde(default)]
    pub bo: f64,
    #[serde(default)]
    pub cash: f64,
    #[serde(default)]
    pub var: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnl: Option<PNL>,
    #[serde(default)]
    pub leverage: f64,
    pub charges: Charges,
    pub total: f64,
}

/// OrderCharges represent an item's response from the Charges calculator API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCharges {
    pub exchange: String,
    #[serde(rename = "tradingsymbol")]
    pub trading_symbol: String,
    pub transaction_type: String,
    pub variety: String,
    pub product: String,
    pub order_type: String,
    pub quantity: f64,
    pub price: f64,
    pub charges: Charges,
}

/// BasketMargins represents response from the Margin Calculator API for Basket orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasketMargins {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<OrderMargins>,
    #[serde(rename = "final")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_margins: Option<OrderMargins>,
    pub orders: Vec<OrderMargins>,
}

/// Parameters for getting order margins
#[derive(Debug, Clone)]
pub struct GetMarginParams {
    pub order_params: Vec<OrderMarginParam>,
    pub compact: bool,
}

/// Parameters for getting basket margins
#[derive(Debug, Clone)]
pub struct GetBasketParams {
    pub order_params: Vec<OrderMarginParam>,
    pub compact: bool,
    pub consider_positions: bool,
}

/// Parameters for getting order charges
#[derive(Debug, Clone)]
pub struct GetChargesParams {
    pub order_params: Vec<OrderChargesParam>,
}

impl KiteConnect {
    /// Get order margins for a list of orders
    pub async fn get_order_margins(
        &self,
        params: GetMarginParams,
    ) -> Result<Vec<OrderMargins>, KiteConnectError> {
        let mut endpoint = Endpoints::ORDER_MARGINS.to_string();
        if params.compact {
            endpoint.push_str("?mode=compact");
        }

        self.post_json(&endpoint, params.order_params).await
    }

    /// Get basket margins for a list of orders
    pub async fn get_basket_margins(
        &self,
        params: GetBasketParams,
    ) -> Result<BasketMargins, KiteConnectError> {
        let mut endpoint = Endpoints::BASKET_MARGINS.to_string();
        let mut query_params = Vec::new();

        if params.compact {
            query_params.push("mode=compact");
        }
        if params.consider_positions {
            query_params.push("consider_positions=true");
        }

        if !query_params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&query_params.join("&"));
        }

        self.post_json(&endpoint, &params.order_params).await
    }

    /// Get order charges for a list of orders
    pub async fn get_order_charges(
        &self,
        params: GetChargesParams,
    ) -> Result<Vec<OrderCharges>, KiteConnectError> {
        self.post_json(Endpoints::ORDER_CHARGES, params.order_params)
            .await
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    constants::Endpoints,
    models::{KiteConnectError, time},
};

/// MFHolding represents an individual mutual fund holding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFHolding {
    pub folio: String,
    pub fund: String,
    pub tradingsymbol: String,
    pub average_price: f64,
    pub last_price: f64,
    pub last_price_date: String,
    pub pnl: f64,
    pub quantity: f64,
    pub pledged_quantity: Option<f64>,
}

/// MFTrade represents an individual trade of a mutual fund holding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFTrade {
    pub fund: String,
    pub tradingsymbol: String,
    pub average_price: f64,
    pub variety: String,
    #[serde(default)]
    pub exchange_timestamp: time::Time,
    pub amount: f64,
    pub folio: String,
    pub quantity: f64,
}

/// MFHoldingBreakdown represents a list of mutual fund holdings.
pub type MFHoldingBreakdown = Vec<MFTrade>;

/// MFHoldings represents a list of mutual fund holdings.
pub type MFHoldings = Vec<MFHolding>;

/// MFOrder represents an individual mutual fund order response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFOrder {
    pub order_id: String,
    pub exchange_order_id: Option<String>,
    pub tradingsymbol: String,
    pub status: String,
    pub status_message: Option<String>,
    pub folio: Option<String>,
    pub fund: String,
    #[serde(default)]
    pub order_timestamp: time::Time,
    #[serde(default)]
    pub exchange_timestamp: Option<time::Time>,
    pub settlement_id: Option<String>,

    pub transaction_type: String,
    pub variety: String,
    pub purchase_type: Option<String>,
    pub quantity: f64,
    pub amount: f64,
    pub last_price: f64,
    pub last_price_date: Option<String>,
    pub average_price: f64,
    pub placed_by: String,
    pub tag: Option<String>,
}

/// MFOrders represents a list of mutual fund orders.
pub type MFOrders = Vec<MFOrder>;

/// MFAllottedISINs represents a list of all ISINs in which at least one allotment is present.
pub type MFAllottedISINs = Vec<String>;

/// MFSIPStepUp represents stepup date and percentage for SIPs.
pub type MFSIPStepUp = HashMap<String, i32>;

/// MFSIP represents an individual mutual fund SIP response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFSIP {
    pub sip_id: String,
    pub tradingsymbol: String,
    pub fund: String,
    pub dividend_type: String,
    pub transaction_type: String,

    pub status: String,
    pub sip_type: String,
    #[serde(default)]
    pub created: time::Time,
    pub frequency: String,
    pub instalment_amount: f64,
    pub instalments: i32,
    #[serde(default)]
    pub last_instalment: time::Time,
    pub pending_instalments: i32,
    pub instalment_day: i32,
    pub completed_instalments: i32,
    pub next_instalment: String,
    pub trigger_price: f64,
    pub step_up: MFSIPStepUp,
    pub tag: Option<String>,

    // Additional fields found in the JSON
    pub sip_reg_num: Option<String>,
}

/// MFSIPs represents a list of mutual fund SIPs.
pub type MFSIPs = Vec<MFSIP>;

/// MFOrderResponse represents the successful order place response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFOrderResponse {
    pub order_id: String,
}

/// MFSIPResponse represents the successful SIP place response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFSIPResponse {
    pub order_id: Option<String>,
    pub sip_id: String,
}

/// MFOrderParams represents parameters for placing an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFOrderParams {
    pub tradingsymbol: Option<String>,
    pub transaction_type: Option<String>,
    pub quantity: Option<f64>,
    pub amount: Option<f64>,
    pub tag: Option<String>,
}

/// MFSIPParams represents parameters for placing a SIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFSIPParams {
    pub tradingsymbol: Option<String>,
    pub amount: Option<f64>,
    pub instalments: Option<i32>,
    pub frequency: Option<String>,
    pub instalment_day: Option<i32>,
    pub initial_amount: Option<f64>,
    pub trigger_price: Option<f64>,
    pub step_up: Option<String>,
    pub sip_type: Option<String>,
    pub tag: Option<String>,
}

/// MFSIPModifyParams represents parameters for modifying a SIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFSIPModifyParams {
    pub amount: Option<f64>,
    pub frequency: Option<String>,
    pub instalment_day: Option<i32>,
    pub instalments: Option<i32>,
    pub step_up: Option<String>,
    pub status: Option<String>,
}

impl KiteConnect {
    /// Gets list of mutual fund orders.
    pub async fn get_mf_orders(&self) -> Result<MFOrders, KiteConnectError> {
        self.get(Endpoints::GET_MF_ORDERS).await
    }

    /// Gets list of mutual fund orders for a custom date range.
    pub async fn get_mf_orders_by_date(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<MFOrders, KiteConnectError> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), from_date.to_string());
        params.insert("to".to_string(), to_date.to_string());

        self.get_with_query(Endpoints::GET_MF_ORDERS, params).await
    }

    /// Gets individual mutual fund order info.
    pub async fn get_mf_order_info(&self, order_id: &str) -> Result<MFOrder, KiteConnectError> {
        let endpoint = &Endpoints::GET_MF_ORDER_INFO.replace("{order_id}", order_id);
        self.get(endpoint).await
    }

    /// Gets list of user mutual fund holdings.
    pub async fn get_mf_holdings(&self) -> Result<MFHoldings, KiteConnectError> {
        self.get(Endpoints::GET_MF_HOLDINGS).await
    }

    /// Gets list of mutual fund SIPs.
    pub async fn get_mf_sips(&self) -> Result<MFSIPs, KiteConnectError> {
        self.get(Endpoints::GET_MF_SIPS).await
    }

    /// Gets individual SIP info.
    pub async fn get_mf_sip_info(&self, sip_id: &str) -> Result<MFSIP, KiteConnectError> {
        let endpoint = &Endpoints::GET_MF_SIP_INFO.replace("{sip_id}", sip_id);
        self.get(endpoint).await
    }

    /// Gets list of user mutual fund allotted ISINs.
    pub async fn get_mf_allotted_isins(&self) -> Result<MFAllottedISINs, KiteConnectError> {
        self.get(Endpoints::GET_MF_ALLOTTED_ISINS).await
    }

    // Deprecated methods for mutual funds.
    // /// Gets individual holding info.
    // pub async fn get_mf_holding_info(
    //     &self,
    //     isin: &str,
    // ) -> Result<MFHoldingBreakdown, KiteConnectError> {
    //     let endpoint = &Endpoints::GET_MF_HOLDING_INFO.replace("{isin}", isin);
    //     self.get(endpoint).await
    // }

    // /// Places a mutual fund order.
    // pub async fn place_mf_order(
    //     &self,
    //     order_params: MFOrderParams,
    // ) -> Result<MFOrderResponse, KiteConnectError> {
    //     self.post_form(Endpoints::PLACE_MF_ORDER, order_params)
    //         .await
    // }

    // /// Cancels a mutual fund order.
    // pub async fn cancel_mf_order(
    //     &self,
    //     order_id: &str,
    // ) -> Result<MFOrderResponse, KiteConnectError> {
    //     let endpoint = &Endpoints::CANCEL_MF_ORDER.replace("{order_id}", order_id);
    //     self.delete(endpoint).await
    // }

    // /// Places a mutual fund SIP order.
    // pub async fn place_mf_sip(
    //     &self,
    //     sip_params: MFSIPParams,
    // ) -> Result<MFSIPResponse, KiteConnectError> {
    //     self.post_form(Endpoints::PLACE_MF_SIP, sip_params).await
    // }

    // /// Modifies a mutual fund SIP.
    // pub async fn modify_mf_sip(
    //     &self,
    //     sip_id: &str,
    //     sip_params: MFSIPModifyParams,
    // ) -> Result<MFSIPResponse, KiteConnectError> {
    //     let endpoint = &Endpoints::MODIFY_MF_SIP.replace("{sip_id}", sip_id);
    //     self.put_form(endpoint, sip_params).await
    // }

    // /// Cancels a mutual fund SIP.
    // pub async fn cancel_mf_sip(&self, sip_id: &str) -> Result<MFSIPResponse, KiteConnectError> {
    //     let endpoint = &Endpoints::CANCEL_MF_SIP.replace("{sip_id}", sip_id);
    //     self.delete(endpoint).await
    // }
}

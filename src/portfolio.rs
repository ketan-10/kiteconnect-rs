use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    constants::{Endpoints, app_constants::*},
    models::{KiteConnectError, time},
};

// MTFHolding represents the mtf details for a holding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTFHolding {
    pub quantity: i32,
    pub used_quantity: i32,
    pub average_price: f64,
    pub value: f64,
    pub initial_margin: f64,
}

// Holding is an individual holdings response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,
    pub isin: String,
    pub product: String,

    pub price: f64,
    pub used_quantity: i32,
    pub quantity: i32,
    pub t1_quantity: i32,
    pub realised_quantity: i32,
    pub authorised_quantity: i32,
    pub authorised_date: time::Time,
    pub opening_quantity: i32,
    pub collateral_quantity: i32,
    pub collateral_type: String,

    pub discrepancy: bool,
    pub average_price: f64,
    pub last_price: f64,
    pub close_price: f64,
    pub pnl: f64,
    pub day_change: f64,
    pub day_change_percentage: f64,

    pub mtf: MTFHolding,
}

// Holdings is a list of holdings
pub type Holdings = Vec<Holding>;

// Position represents an individual position response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,
    pub product: String,

    pub quantity: i32,
    pub overnight_quantity: i32,
    pub multiplier: f64,

    pub average_price: f64,
    pub close_price: f64,
    pub last_price: f64,
    pub value: f64,
    pub pnl: f64,
    pub m2m: f64,
    pub unrealised: f64,
    pub realised: f64,

    pub buy_quantity: i32,
    pub buy_price: f64,
    pub buy_value: f64,
    pub buy_m2m: f64,

    pub sell_quantity: i32,
    pub sell_price: f64,
    pub sell_value: f64,
    pub sell_m2m: f64,

    pub day_buy_quantity: i32,
    pub day_buy_price: f64,
    pub day_buy_value: f64,

    pub day_sell_quantity: i32,
    pub day_sell_price: f64,
    pub day_sell_value: f64,
}

// Positions represents a list of net and day positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Positions {
    pub net: Vec<Position>,
    pub day: Vec<Position>,
}

// ConvertPositionParams represents the input params for a position conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertPositionParams {
    pub exchange: String,
    pub tradingsymbol: String,
    pub old_product: String,
    pub new_product: String,
    pub position_type: String,
    pub transaction_type: String,
    pub quantity: i32,
}

// AuctionInstrument represents the auction instrument available for a auction session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionInstrument {
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,
    pub isin: String,
    pub product: String,
    pub price: f64,
    pub quantity: i32,
    pub t1_quantity: i32,
    pub realised_quantity: i32,
    pub authorised_quantity: i32,
    pub authorised_date: String,
    pub opening_quantity: i32,
    pub collateral_quantity: i32,
    pub collateral_type: String,
    pub discrepancy: bool,
    pub average_price: f64,
    pub last_price: f64,
    pub close_price: f64,
    pub pnl: f64,
    pub day_change: f64,
    pub day_change_percentage: f64,
    pub auction_number: String,
}

// HoldingsAuthInstruments represents the instruments and respective quantities for
// use within the holdings auth initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsAuthInstruments {
    pub isin: String,
    pub quantity: f64,
}

// HoldingAuthParams represents the inputs for initiating holdings authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingAuthParams {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub transfer_type: String,
    pub exec_date: String,
    // Instruments are optional
    pub instruments: Option<Vec<HoldingsAuthInstruments>>,
}

// HoldingsAuthResp represents the response from initiating holdings authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsAuthResp {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
}

impl KiteConnect {
    /// Get a list of holdings
    pub async fn get_holdings(&self) -> Result<Holdings, KiteConnectError> {
        self.get(Endpoints::GET_HOLDINGS).await
    }

    /// Get auction instruments - retrieves list of available instruments for a auction session
    pub async fn get_auction_instruments(
        &self,
    ) -> Result<Vec<AuctionInstrument>, KiteConnectError> {
        self.get(Endpoints::AUCTION_INSTRUMENTS).await
    }

    /// Get user positions
    pub async fn get_positions(&self) -> Result<Positions, KiteConnectError> {
        self.get(Endpoints::GET_POSITIONS).await
    }

    /// Convert position's product type
    pub async fn convert_position(
        &self,
        position_params: ConvertPositionParams,
    ) -> Result<bool, KiteConnectError> {
        // For position conversion, we expect an empty response on success
        match self
            .put_form::<serde_json::Value, _>(Endpoints::CONVERT_POSITION, position_params)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Initiate holdings authorization flow
    ///
    /// It accepts an optional list of HoldingsAuthInstruments which can be used to specify
    /// a set of ISINs with their respective quantities. Since, the isin and quantity pairs
    /// here are optional, you can provide it as None. If they're provided, authorization
    /// is sought only for those instruments and otherwise, the entire holdings is presented
    /// for authorization. The response contains the RequestID which can then be used to
    /// redirect the user in a web view. The client forms and returns the formed RedirectURL as well.
    pub async fn initiate_holdings_auth(
        &self,
        auth_params: HoldingAuthParams,
    ) -> Result<HoldingsAuthResp, KiteConnectError> {
        let mut params = HashMap::new();

        if !auth_params.auth_type.is_empty() {
            params.insert("type".to_string(), auth_params.auth_type);
        }

        if !auth_params.transfer_type.is_empty() {
            params.insert("transfer_type".to_string(), auth_params.transfer_type);
        }

        if !auth_params.exec_date.is_empty() {
            params.insert("exec_date".to_string(), auth_params.exec_date);
        }

        // Handle optional instruments
        if let Some(instruments) = auth_params.instruments {
            for instrument in instruments {
                params.insert("isin".to_string(), instrument.isin);
                params.insert("quantity".to_string(), instrument.quantity.to_string());
            }
        }

        let mut resp: HoldingsAuthResp = self
            .post_form(Endpoints::INIT_HOLDINGS_AUTH, params)
            .await?;

        let login_url = format!(
            "{}/connect/portfolio/authorise/holdings/{}/{}",
            KITE_BASE_URL, &self.api_key, &resp.request_id
        );
        // Form and set the URL in the response
        resp.redirect_url = Some(login_url);

        Ok(resp)
    }
}

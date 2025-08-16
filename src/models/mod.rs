use serde::{Deserialize, Serialize};

pub mod error;
pub mod time;

pub use error::{KiteConnectError, KiteConnectErrorKind, KiteError};

// OHLC represents OHLC packets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OHLC {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_token: Option<u32>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

// DepthItem represents a single market depth entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DepthItem {
    pub price: f64,
    pub quantity: u32,
    pub orders: u32,
}

impl Default for DepthItem {
    fn default() -> Self {
        Self {
            price: 0.0,
            quantity: 0,
            orders: 0,
        }
    }
}

// Depth represents a group of buy/sell market depths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Depth {
    pub buy: [DepthItem; 5],
    pub sell: [DepthItem; 5],
}

impl Default for Depth {
    fn default() -> Self {
        Self {
            buy: [DepthItem::default(); 5],
            sell: [DepthItem::default(); 5],
        }
    }
}

// Tick represents a single packet in the market feed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    pub mode: String,
    pub instrument_token: u32,
    pub is_tradable: bool,
    pub is_index: bool,

    // Timestamp represents Exchange timestamp
    pub timestamp: time::Time,
    pub last_trade_time: time::Time,
    pub last_price: f64,
    pub last_traded_quantity: u32,
    pub total_buy_quantity: u32,
    pub total_sell_quantity: u32,
    pub volume_traded: u32,
    pub total_buy: u32,
    pub total_sell: u32,
    pub average_trade_price: f64,
    pub oi: u32,
    pub oi_day_high: u32,
    pub oi_day_low: u32,
    pub net_change: f64,

    pub ohlc: OHLC,
    pub depth: Depth,
}

impl Default for Tick {
    fn default() -> Self {
        Self {
            mode: String::new(),
            instrument_token: 0,
            is_tradable: false,
            is_index: false,
            timestamp: time::Time::default(),
            last_trade_time: time::Time::default(),
            last_price: 0.0,
            last_traded_quantity: 0,
            total_buy_quantity: 0,
            total_sell_quantity: 0,
            volume_traded: 0,
            total_buy: 0,
            total_sell: 0,
            average_trade_price: 0.0,
            oi: 0,
            oi_day_high: 0,
            oi_day_low: 0,
            net_change: 0.0,
            ohlc: OHLC {
                instrument_token: None,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
            },
            depth: Depth::default(),
        }
    }
}

// Order represents an order structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub account_id: String,
    pub placed_by: String,

    pub order_id: String,
    pub exchange_order_id: String,
    pub parent_order_id: String,
    pub status: String,
    pub status_message: String,
    pub status_message_raw: String,
    pub order_timestamp: time::Time,
    pub exchange_update_timestamp: time::Time,
    pub exchange_timestamp: time::Time,
    pub variety: String,
    pub modified: bool,
    pub meta: serde_json::Map<String, serde_json::Value>,

    pub exchange: String,
    pub tradingsymbol: String,
    pub instrument_token: u32,

    pub order_type: String,
    pub transaction_type: String,
    pub validity: String,
    pub validity_ttl: i32,
    pub product: String,
    pub quantity: f64,
    pub disclosed_quantity: f64,
    pub price: f64,
    pub trigger_price: f64,

    pub average_price: f64,
    pub filled_quantity: f64,
    pub pending_quantity: f64,
    pub cancelled_quantity: f64,

    pub auction_number: String,

    pub tag: String,
    pub tags: Vec<String>,
}

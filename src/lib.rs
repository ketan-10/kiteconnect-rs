pub mod compat;
pub mod connect;

pub mod http;
pub mod margins;
pub mod markets;
pub mod mf;

pub mod alerts;
pub mod orders;
pub mod portfolio;
pub mod ticker;
pub mod users;

pub use connect::{KiteConnect, KiteConnectBuilder};
pub use models::*;
pub use ticker::{Mode, Ticker, TickerBuilder, TickerError, TickerEvent};

// Re-export order types
pub use orders::{Order, OrderParams, OrderResponse, Orders, Trade, Trades};

pub mod constants;
#[path = "models/mod.rs"]
pub mod models;
pub use constants::Endpoints;
pub use constants::Labels;
pub use constants::app_constants::*;

// Re-export portfolio types
pub use portfolio::{
    AuctionInstrument, ConvertPositionParams, Holding, HoldingAuthParams, Holdings,
    HoldingsAuthInstruments, HoldingsAuthResp, MTFHolding, Position, Positions,
};

// Re-export user types
pub use users::{
    AllMargins, AvailableMargins, Bank, FullUserMeta, FullUserProfile, Margins, UsedMargins,
    UserMeta, UserProfile, UserSession, UserSessionTokens,
};

// Re-export mutual fund types
pub use mf::{
    MFAllottedISINs, MFHolding, MFHoldingBreakdown, MFHoldings, MFOrder, MFOrderParams,
    MFOrderResponse, MFOrders, MFSIP, MFSIPModifyParams, MFSIPParams, MFSIPResponse, MFSIPStepUp,
    MFSIPs, MFTrade,
};

// Re-export margins types
pub use margins::{
    BasketMargins, Charges, GST, GetBasketParams, GetChargesParams, GetMarginParams, OrderCharges,
    OrderChargesParam, OrderMarginParam, OrderMargins, PNL,
};

// Re-export market data types
pub use markets::{
    HistoricalData, HistoricalDataParams, Instrument, Instruments, MFInstrument, MFInstruments,
    Quote, QuoteData, QuoteLTP, QuoteLTPData, QuoteOHLC, QuoteOHLCData,
};

// Re-export alerts types
pub use alerts::{
    Alert, AlertHistory, AlertHistoryMeta, AlertOperator, AlertOrderParams, AlertParams,
    AlertStatus, AlertType, Basket, BasketItem, OrderGTTParams,
};

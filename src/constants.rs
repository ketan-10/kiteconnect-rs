pub mod app_constants {
    use std::time::Duration;

    pub const DEFAULT_BASE_URL: &str = "https://api.kite.trade";
    pub const KITE_BASE_URL: &str = "https://kite.zerodha.com";

    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(7);

    pub const KITE_HEADER_VERSION: &str = "3";
    pub const KITE_CONNECT_RS_NAME: &str = "kiteconnect-rs";
    pub const KITE_CONNECT_RS_VERSION: &str = "4.0.2";
}

// API endpoints
pub struct Endpoints;

impl Endpoints {
    pub const LOGIN_URL: &'static str = "/connect/login";
    pub const SESSION_GENERATE: &'static str = "/session/token";
    pub const INVALIDATE_TOKEN: &'static str = "/session/token";
    pub const RENEW_ACCESS: &'static str = "/session/refresh_token";
    pub const USER_PROFILE: &'static str = "/user/profile";
    pub const USER_FULL_PROFILE: &'static str = "/user/profile/full";
    pub const USER_MARGINS: &'static str = "/user/margins";
    pub const USER_MARGINS_SEGMENT: &'static str = "/user/margins/{segment}";

    // Portfolio endpoints
    pub const GET_HOLDINGS: &'static str = "/portfolio/holdings";
    pub const GET_POSITIONS: &'static str = "/portfolio/positions";
    pub const CONVERT_POSITION: &'static str = "/portfolio/positions";
    pub const AUCTION_INSTRUMENTS: &'static str = "/portfolio/holdings/auctions";
    pub const INIT_HOLDINGS_AUTH: &'static str = "/portfolio/holdings/authorise";

    // Order endpoints
    pub const GET_ORDERS: &'static str = "/orders";
    pub const GET_TRADES: &'static str = "/trades";
    pub const GET_ORDER_HISTORY: &'static str = "/orders/{order_id}";
    pub const GET_ORDER_TRADES: &'static str = "/orders/{order_id}/trades";
    pub const PLACE_ORDER: &'static str = "/orders/{variety}";
    pub const MODIFY_ORDER: &'static str = "/orders/{variety}/{order_id}";
    pub const CANCEL_ORDER: &'static str = "/orders/{variety}/{order_id}";

    // Mutual Fund endpoints
    pub const GET_MF_ORDERS: &'static str = "/mf/orders";
    pub const GET_MF_ORDER_INFO: &'static str = "/mf/orders/{order_id}";
    pub const PLACE_MF_ORDER: &'static str = "/mf/orders";
    pub const CANCEL_MF_ORDER: &'static str = "/mf/orders/{order_id}";
    pub const GET_MF_SIPS: &'static str = "/mf/sips";
    pub const GET_MF_SIP_INFO: &'static str = "/mf/sips/{sip_id}";
    pub const PLACE_MF_SIP: &'static str = "/mf/sips";
    pub const MODIFY_MF_SIP: &'static str = "/mf/sips/{sip_id}";
    pub const CANCEL_MF_SIP: &'static str = "/mf/sips/{sip_id}";
    pub const GET_MF_HOLDINGS: &'static str = "/mf/holdings";
    pub const GET_MF_HOLDING_INFO: &'static str = "/mf/holdings/{isin}";
    pub const GET_MF_ALLOTTED_ISINS: &'static str = "/mf/allotments";

    // Margin endpoints
    pub const ORDER_MARGINS: &'static str = "/margins/orders";
    pub const BASKET_MARGINS: &'static str = "/margins/basket";
    pub const ORDER_CHARGES: &'static str = "/charges/orders";

    // Market data endpoints
    pub const GET_QUOTE: &'static str = "/quote";
    pub const GET_LTP: &'static str = "/quote/ltp";
    pub const GET_OHLC: &'static str = "/quote/ohlc";
    pub const GET_INSTRUMENTS: &'static str = "/instruments";
    pub const GET_MF_INSTRUMENTS: &'static str = "/mf/instruments";
    pub const GET_INSTRUMENTS_EXCHANGE: &'static str = "/instruments/{exchange}";
    pub const GET_HISTORICAL: &'static str =
        "/instruments/historical/{instrument_token}/{interval}";

    // Alerts endpoints
    pub const ALERTS_URL: &'static str = "/alerts";
    pub const ALERT_URL: &'static str = "/alerts/{alert_id}";
    pub const GET_ALERT_HISTORY: &'static str = "/alerts/{alert_id}/history";
}

pub struct Labels;

impl Labels {
    // Order varieties
    pub const VARIETY_REGULAR: &str = "regular";
    pub const VARIETY_AMO: &str = "amo";
    pub const VARIETY_ICEBERG: &str = "iceberg";
    pub const VARIETY_BRACKET: &str = "bo";
    pub const VARIETY_COVER: &str = "co";
    pub const VARIETY_AUCTION: &str = "auction";

    // Order types
    pub const ORDER_TYPE_MARKET: &str = "MARKET";
    pub const ORDER_TYPE_LIMIT: &str = "LIMIT";
    pub const ORDER_TYPE_SL: &str = "SL";
    pub const ORDER_TYPE_SL_M: &str = "SL-M";

    // Transaction types
    pub const TRANSACTION_TYPE_BUY: &str = "BUY";
    pub const TRANSACTION_TYPE_SELL: &str = "SELL";

    // Products
    pub const PRODUCT_CNC: &str = "CNC";
    pub const PRODUCT_MIS: &str = "MIS";
    pub const PRODUCT_NRML: &str = "NRML";
    pub const PRODUCT_BO: &str = "BO";
    pub const PRODUCT_CO: &str = "CO";

    // Validity
    pub const VALIDITY_DAY: &str = "DAY";
    pub const VALIDITY_IOC: &str = "IOC";
    pub const VALIDITY_TTL: &str = "TTL";

    // Exchanges
    pub const EXCHANGE_NSE: &str = "NSE";
    pub const EXCHANGE_BSE: &str = "BSE";
    pub const EXCHANGE_NFO: &str = "NFO";
    pub const EXCHANGE_BFO: &str = "BFO";
    pub const EXCHANGE_MCX: &str = "MCX";
    pub const EXCHANGE_CDS: &str = "CDS";

    // Constants for Holdings Auth types
    pub const HOL_AUTH_TYPE_MF: &str = "mf";
    pub const HOL_AUTH_TYPE_EQUITY: &str = "equity";

    pub const HOL_AUTH_TRANSFER_TYPE_PRE_TRADE: &str = "pre";
    pub const HOL_AUTH_TRANSFER_TYPE_POST_TRADE: &str = "post";
    pub const HOL_AUTH_TRANSFER_TYPE_OFF_MARKET: &str = "off";
    pub const HOL_AUTH_TRANSFER_TYPE_GIFT: &str = "gift";
}

//! WASM-specific tests for kiteconnect-rs
//!
//! Run with Node.js: wasm-pack test --node -- --test wasm
//! Run in browser:   wasm-pack test --headless --chrome -- --test wasm
//!                   (requires uncommenting `run_in_browser` config below)

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

// Note: Remove this line to run tests in Node.js instead of browser
// wasm_bindgen_test_configure!(run_in_browser);

use kiteconnect_rs::compat::{sleep, timeout, TimeoutError};
use kiteconnect_rs::{KiteConnect, TickerBuilder};
use web_time::Duration;

// ============================================================================
// Compat Module Tests
// ============================================================================

#[wasm_bindgen_test]
async fn test_sleep() {
    let start = web_time::Instant::now();
    sleep(Duration::from_millis(100)).await;
    let elapsed = start.elapsed();

    // Should have slept for at least 100ms (with some tolerance)
    assert!(elapsed >= Duration::from_millis(90), "Sleep was too short: {:?}", elapsed);
}

#[wasm_bindgen_test]
async fn test_timeout_success() {
    // Task that completes before timeout
    let result = timeout(Duration::from_millis(500), async {
        sleep(Duration::from_millis(50)).await;
        42
    }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[wasm_bindgen_test]
async fn test_timeout_expired() {
    // Task that takes longer than timeout
    let result: Result<(), TimeoutError> = timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(500)).await;
    }).await;

    assert!(result.is_err());
}

// ============================================================================
// KiteConnect Builder Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_kite_connect_builder() {
    let kite = KiteConnect::builder("test_api_key")
        .build();

    assert!(kite.is_ok());
}

#[wasm_bindgen_test]
fn test_kite_connect_with_access_token() {
    let kite = KiteConnect::builder("test_api_key")
        .access_token("test_token")
        .build();

    assert!(kite.is_ok());
}

#[wasm_bindgen_test]
fn test_kite_connect_login_url() {
    let kite = KiteConnect::builder("my_api_key")
        .build()
        .unwrap();

    let login_url = kite.get_login_url();
    assert!(login_url.contains("my_api_key"));
    assert!(login_url.contains("kite.zerodha.com"));
}

// ============================================================================
// Ticker Builder Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_ticker_builder() {
    let result = TickerBuilder::new("test_api_key", "test_access_token")
        .build();

    // Ticker should build successfully
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_ticker_builder_with_options() {
    let result = TickerBuilder::new("test_api_key", "test_access_token")
        .auto_reconnect(true)
        .reconnect_max_retries(5)
        .reconnect_max_delay(Duration::from_secs(30))
        .build();

    // Should build with custom options
    assert!(result.is_ok());
}

// ============================================================================
// Constants Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_constants_available() {
    use kiteconnect_rs::{Labels, Endpoints};

    // Verify constants are accessible
    assert_eq!(Labels::EXCHANGE_NSE, "NSE");
    assert_eq!(Labels::ORDER_TYPE_MARKET, "MARKET");
    assert_eq!(Labels::TRANSACTION_TYPE_BUY, "BUY");
    assert!(!Endpoints::LOGIN_URL.is_empty());
}

// ============================================================================
// Duration/Time Tests (web-time compatibility)
// ============================================================================

#[wasm_bindgen_test]
fn test_web_time_duration() {
    let duration = Duration::from_secs(5);
    assert_eq!(duration.as_secs(), 5);
    assert_eq!(duration.as_millis(), 5000);
}

#[wasm_bindgen_test]
fn test_web_time_instant() {
    let now = web_time::Instant::now();
    let later = web_time::Instant::now();

    // Later should be >= now
    assert!(later >= now);
}

// ============================================================================
// Ticker Parsing Tests (cross-platform, reused from ticker_tests.rs)
// ============================================================================

use base64::{Engine as _, engine::general_purpose};
use kiteconnect_rs::{DepthItem, Mode, Ticker};

// Packet data embedded at compile time from files (works in both native and WASM)
const TICKER_QUOTE_PACKET: &str = include_str!("mocks/ticker_quote.packet");
const TICKER_FULL_PACKET: &str = include_str!("mocks/ticker_full.packet");

fn decode_packet(base64_data: &str) -> Vec<u8> {
    general_purpose::STANDARD.decode(base64_data.trim()).unwrap()
}

#[wasm_bindgen_test]
fn test_packet_parsing_ltp() {
    // Test LTP packet parsing
    let data = vec![
        0x00, 0x06, 0x3a, 0x01, // instrument token: 408065
        0x00, 0x02, 0x66, 0x83, // last price: 157315 (1573.15 after conversion)
    ];

    let result = Ticker::parse_packet(&data);
    assert!(result.is_ok());

    let tick = result.unwrap();
    assert_eq!(tick.instrument_token, 408065);
    assert_eq!(tick.mode, "ltp");
    assert_eq!(tick.last_price, 1573.15);
}

#[wasm_bindgen_test]
fn test_price_conversion() {
    // Test NSE/BSE equity price conversion (divide by 100)
    let price = Ticker::convert_price(1, 157315);
    assert_eq!(price, 1573.15);

    // Test NSE CD price conversion (divide by 10,000,000)
    let price = Ticker::convert_price(3, 157315000);
    assert_eq!(price, 15.7315);

    // Test BSE CD price conversion (divide by 10,000)
    let price = Ticker::convert_price(6, 157315);
    assert_eq!(price, 15.7315);
}

#[wasm_bindgen_test]
fn test_split_packets() {
    // Create test data with 2 packets
    let mut data = vec![0x00, 0x02]; // 2 packets

    // First packet: 8 bytes (LTP packet)
    data.extend_from_slice(&[0x00, 0x08]); // packet length
    data.extend_from_slice(&[0x00, 0x06, 0x37, 0x81]); // instrument token
    data.extend_from_slice(&[0x00, 0x02, 0x66, 0x7B]); // price data

    // Second packet: 8 bytes (another LTP packet)
    data.extend_from_slice(&[0x00, 0x08]); // packet length
    data.extend_from_slice(&[0x00, 0x0B, 0x44, 0x41]); // different instrument token
    data.extend_from_slice(&[0x00, 0x03, 0x88, 0x9C]); // different price data

    let packets = Ticker::split_packets(&data);
    assert_eq!(packets.len(), 2);
    assert_eq!(packets[0].len(), 8);
    assert_eq!(packets[1].len(), 8);
}

#[wasm_bindgen_test]
fn test_mode_display() {
    assert_eq!(Mode::LTP.to_string(), "ltp");
    assert_eq!(Mode::Quote.to_string(), "quote");
    assert_eq!(Mode::Full.to_string(), "full");
}

#[wasm_bindgen_test]
fn test_parse_quote_packet() {
    let packet_data = decode_packet(TICKER_QUOTE_PACKET);

    let result = Ticker::parse_packet(&packet_data);
    assert!(result.is_ok());

    let tick = result.unwrap();

    // Expected values from the Go test case
    assert_eq!(tick.mode, "quote");
    assert_eq!(tick.instrument_token, 408065);
    assert_eq!(tick.is_tradable, true);
    assert_eq!(tick.is_index, false);
    assert_eq!(tick.last_price, 1573.15);
    assert_eq!(tick.last_traded_quantity, 1);
    assert_eq!(tick.total_buy_quantity, 256511);
    assert_eq!(tick.total_sell_quantity, 360503);
    assert_eq!(tick.volume_traded, 1175986);
    assert_eq!(tick.total_buy, 0);
    assert_eq!(tick.total_sell, 0);
    assert_eq!(tick.average_trade_price, 1570.33);
    assert_eq!(tick.oi, 0);
    assert_eq!(tick.oi_day_high, 0);
    assert_eq!(tick.oi_day_low, 0);

    // OHLC values
    assert_eq!(tick.ohlc.open, 1569.15);
    assert_eq!(tick.ohlc.high, 1575.0);
    assert_eq!(tick.ohlc.low, 1561.05);
    assert_eq!(tick.ohlc.close, 1567.8);

    // Net change calculation (last_price - close_price)
    let expected_net_change = tick.last_price - tick.ohlc.close;
    assert!((tick.net_change - expected_net_change).abs() < 0.01);

    // Depth should be empty for quote mode (if not full packet)
    assert_eq!(tick.depth.buy[0].price, 0.0);
    assert_eq!(tick.depth.sell[0].price, 0.0);
}

#[wasm_bindgen_test]
fn test_parse_full_packet() {
    let packet_data = decode_packet(TICKER_FULL_PACKET);

    let result = Ticker::parse_packet(&packet_data);
    assert!(result.is_ok());

    let tick = result.unwrap();

    // Expected values from the Go test case
    assert_eq!(tick.mode, "full");
    assert_eq!(tick.instrument_token, 408065);
    assert_eq!(tick.is_tradable, true);
    assert_eq!(tick.is_index, false);
    assert_eq!(tick.last_price, 1573.7);
    assert_eq!(tick.last_traded_quantity, 7);
    assert_eq!(tick.total_buy_quantity, 256443);
    assert_eq!(tick.total_sell_quantity, 363009);
    assert_eq!(tick.volume_traded, 1192471);
    assert_eq!(tick.total_buy, 0);
    assert_eq!(tick.total_sell, 0);
    assert_eq!(tick.average_trade_price, 1570.37);
    assert_eq!(tick.oi, 0);
    assert_eq!(tick.oi_day_high, 0);
    assert_eq!(tick.oi_day_low, 0);

    // OHLC values
    assert_eq!(tick.ohlc.open, 1569.15);
    assert_eq!(tick.ohlc.high, 1575.0);
    assert_eq!(tick.ohlc.low, 1561.05);
    assert_eq!(tick.ohlc.close, 1567.8);

    // Net change should be approximately 5.9
    assert!((tick.net_change - 5.9).abs() < 0.1);

    // Timestamp and LastTradeTime should be 1625461887 (Unix timestamp)
    if let Some(dt) = tick.timestamp.as_datetime() {
        assert_eq!(dt.timestamp(), 1625461887);
    }
    if let Some(dt) = tick.last_trade_time.as_datetime() {
        assert_eq!(dt.timestamp(), 1625461887);
    }

    // Check depth data - Buy side
    let expected_buy_depth = [
        DepthItem { price: 1573.4, quantity: 5, orders: 1 },
        DepthItem { price: 1573.0, quantity: 140, orders: 2 },
        DepthItem { price: 1572.95, quantity: 2, orders: 1 },
        DepthItem { price: 1572.9, quantity: 219, orders: 7 },
        DepthItem { price: 1572.85, quantity: 50, orders: 1 },
    ];

    for (i, expected) in expected_buy_depth.iter().enumerate() {
        assert_eq!(tick.depth.buy[i].price, expected.price);
        assert_eq!(tick.depth.buy[i].quantity, expected.quantity);
        assert_eq!(tick.depth.buy[i].orders, expected.orders);
    }

    // Check depth data - Sell side
    let expected_sell_depth = [
        DepthItem { price: 1573.7, quantity: 172, orders: 3 },
        DepthItem { price: 1573.75, quantity: 44, orders: 3 },
        DepthItem { price: 1573.85, quantity: 302, orders: 3 },
        DepthItem { price: 1573.9, quantity: 141, orders: 2 },
        DepthItem { price: 1573.95, quantity: 724, orders: 5 },
    ];

    for (i, expected) in expected_sell_depth.iter().enumerate() {
        assert_eq!(tick.depth.sell[i].price, expected.price);
        assert_eq!(tick.depth.sell[i].quantity, expected.quantity);
        assert_eq!(tick.depth.sell[i].orders, expected.orders);
    }
}

#[wasm_bindgen_test]
fn test_parse_binary_with_multiple_packets() {
    let quote_data = decode_packet(TICKER_QUOTE_PACKET);
    let full_data = decode_packet(TICKER_FULL_PACKET);

    // Create a combined packet with 2 packets
    let mut combined_data = vec![0x00, 0x02]; // 2 packets

    // First packet (quote)
    combined_data.extend_from_slice(&(quote_data.len() as u16).to_be_bytes());
    combined_data.extend_from_slice(&quote_data);

    // Second packet (full)
    combined_data.extend_from_slice(&(full_data.len() as u16).to_be_bytes());
    combined_data.extend_from_slice(&full_data);

    let result = Ticker::parse_binary(&combined_data);
    assert!(result.is_ok());

    let ticks = result.unwrap();
    assert_eq!(ticks.len(), 2);

    // First tick should be quote mode
    assert_eq!(ticks[0].mode, "quote");
    assert_eq!(ticks[0].instrument_token, 408065);

    // Second tick should be full mode
    assert_eq!(ticks[1].mode, "full");
    assert_eq!(ticks[1].instrument_token, 408065);
}

#[wasm_bindgen_test]
fn test_segment_detection() {
    // Test different segment detection
    let nse_cm_token = 408065; // NSE CM
    let indices_token = 0x09; // INDICES

    // NSE CM token
    let seg = nse_cm_token & 0xFF;
    assert_ne!(seg, 9); // Not indices

    // Indices token
    let seg = indices_token & 0xFF;
    assert_eq!(seg, 9); // Is indices
}

#[wasm_bindgen_test]
fn test_ticker_creation() {
    let (ticker, handle) = Ticker::new("test_api_key".to_string(), "test_access_token".to_string());
    // Basic creation test passes if no panic occurs
    // Verify handle can clone event receiver
    let _rx = handle.subscribe_events();
    drop(ticker);
}

#[wasm_bindgen_test]
fn test_reconnect_delay_validation() {
    let (mut ticker, _handle) = Ticker::new("test_api_key".to_string(), "test_access_token".to_string());

    // Test that setting delay below minimum fails
    let result = ticker.set_reconnect_max_delay(Duration::from_millis(1000));
    assert!(result.is_err());

    // Test that setting valid delay succeeds
    let result = ticker.set_reconnect_max_delay(Duration::from_millis(10000));
    assert!(result.is_ok());
}

// ============================================================================
// HTTP API Response Parsing Tests
// ============================================================================
// These tests verify that HTTP API response types can be correctly parsed
// in WASM environment, ensuring JSON deserialization works cross-platform.

use kiteconnect_rs::{
    Holdings, Positions, Orders, Trades,
    Quote, QuoteLTP, QuoteOHLC, OrderParams, ConvertPositionParams,
};

// Embed mock JSON responses at compile time
const POSITIONS_JSON: &str = include_str!("mocks/positions.json");
const HOLDINGS_JSON: &str = include_str!("mocks/holdings.json");
const ORDERS_JSON: &str = include_str!("mocks/orders.json");
const TRADES_JSON: &str = include_str!("mocks/trades.json");
const QUOTE_JSON: &str = include_str!("mocks/quote.json");
const LTP_JSON: &str = include_str!("mocks/ltp.json");
const OHLC_JSON: &str = include_str!("mocks/ohlc.json");

// Helper to extract data from API response wrapper
fn extract_data<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    use serde::de::Error;
    let wrapper: serde_json::Value = serde_json::from_str(json)?;
    let data = wrapper.get("data").ok_or_else(|| {
        serde_json::Error::custom("Missing 'data' field")
    })?;
    serde_json::from_value(data.clone())
}

#[wasm_bindgen_test]
fn test_parse_positions() {
    let result: Result<Positions, _> = extract_data(POSITIONS_JSON);
    assert!(result.is_ok(), "Failed to parse positions: {:?}", result.err());

    let positions = result.unwrap();

    // Verify net positions
    assert!(!positions.net.is_empty(), "Net positions should not be empty");
    let first_net = &positions.net[0];
    assert_eq!(first_net.tradingsymbol, "LEADMINI17DECFUT");
    assert_eq!(first_net.exchange, "MCX");
    assert_eq!(first_net.instrument_token, 53496327);
    assert_eq!(first_net.product, "NRML");
    assert_eq!(first_net.quantity, 1);
    assert_eq!(first_net.multiplier, 1000.0);

    // Verify day positions
    assert!(!positions.day.is_empty(), "Day positions should not be empty");
    let first_day = &positions.day[0];
    assert_eq!(first_day.tradingsymbol, "GOLDGUINEA17DECFUT");
}

#[wasm_bindgen_test]
fn test_parse_holdings() {
    let result: Result<Holdings, _> = extract_data(HOLDINGS_JSON);
    assert!(result.is_ok(), "Failed to parse holdings: {:?}", result.err());

    let holdings = result.unwrap();

    assert!(!holdings.is_empty(), "Holdings should not be empty");
    let first = &holdings[0];
    assert_eq!(first.tradingsymbol, "AARON");
    assert_eq!(first.exchange, "NSE");
    assert_eq!(first.instrument_token, 263681);
    assert_eq!(first.isin, "INE721Z01010");
    assert_eq!(first.product, "CNC");
    assert_eq!(first.quantity, 1);
    assert_eq!(first.average_price, 161.0);

    // Verify MTF data
    assert_eq!(first.mtf.quantity, 1000);
    assert_eq!(first.mtf.average_price, 100.0);
    assert_eq!(first.mtf.value, 100000.0);
}

#[wasm_bindgen_test]
fn test_parse_orders() {
    let result: Result<Orders, _> = extract_data(ORDERS_JSON);
    assert!(result.is_ok(), "Failed to parse orders: {:?}", result.err());

    let orders = result.unwrap();

    assert!(!orders.is_empty(), "Orders should not be empty");

    // Find a completed order
    let completed = orders.iter().find(|o| o.status == "COMPLETE");
    assert!(completed.is_some(), "Should have at least one completed order");
    let order = completed.unwrap();
    assert!(!order.order_id.is_empty());
    assert!(!order.placed_by.is_empty());

    // Find a rejected order with status message
    let rejected = orders.iter().find(|o| o.status == "REJECTED");
    assert!(rejected.is_some(), "Should have a rejected order");
    let rej = rejected.unwrap();
    assert!(rej.status_message.is_some());
}

#[wasm_bindgen_test]
fn test_parse_orders_with_tags() {
    let result: Result<Orders, _> = extract_data(ORDERS_JSON);
    let orders = result.unwrap();

    // Find order with tags
    let with_tags = orders.iter().find(|o| o.tags.as_ref().map(|t| !t.is_empty()).unwrap_or(false));
    assert!(with_tags.is_some(), "Should have at least one order with tags");

    let order = with_tags.unwrap();
    let tags = order.tags.as_ref().unwrap();
    assert!(!tags.is_empty());
}

#[wasm_bindgen_test]
fn test_parse_orders_iceberg() {
    let result: Result<Orders, _> = extract_data(ORDERS_JSON);
    let orders = result.unwrap();

    // Find iceberg order
    let iceberg = orders.iter().find(|o| o.variety == "iceberg");
    assert!(iceberg.is_some(), "Should have an iceberg order");

    let order = iceberg.unwrap();
    assert!(order.meta.contains_key("iceberg"));
}

#[wasm_bindgen_test]
fn test_parse_trades() {
    let result: Result<Trades, _> = extract_data(TRADES_JSON);
    assert!(result.is_ok(), "Failed to parse trades: {:?}", result.err());

    let trades = result.unwrap();

    assert!(!trades.is_empty(), "Trades should not be empty");
    let first = &trades[0];
    assert!(!first.trade_id.is_empty());
    assert!(!first.order_id.is_empty());
    assert!(!first.exchange.is_empty());
    assert!(!first.tradingsymbol.is_empty());
    assert!(first.average_price > 0.0);
    assert!(first.quantity > 0.0);
}

#[wasm_bindgen_test]
fn test_parse_quote() {
    let result: Result<Quote, _> = extract_data(QUOTE_JSON);
    assert!(result.is_ok(), "Failed to parse quote: {:?}", result.err());

    let quote = result.unwrap();

    // Quote is a HashMap<String, QuoteData>
    assert!(quote.contains_key("NSE:INFY"), "Should have NSE:INFY quote");

    let infy = quote.get("NSE:INFY").unwrap();
    assert_eq!(infy.instrument_token, 408065);
    assert!(infy.last_price > 0.0);
    assert!(infy.ohlc.open > 0.0);
    assert!(infy.ohlc.high >= infy.ohlc.low);
}

#[wasm_bindgen_test]
fn test_parse_ltp() {
    let result: Result<QuoteLTP, _> = extract_data(LTP_JSON);
    assert!(result.is_ok(), "Failed to parse LTP: {:?}", result.err());

    let ltp = result.unwrap();

    // LTP should have at least one entry
    assert!(!ltp.is_empty(), "LTP should not be empty");

    // Verify structure
    for (symbol, data) in &ltp {
        assert!(!symbol.is_empty());
        assert!(data.instrument_token > 0);
        assert!(data.last_price >= 0.0);
    }
}

#[wasm_bindgen_test]
fn test_parse_ohlc() {
    let result: Result<QuoteOHLC, _> = extract_data(OHLC_JSON);
    assert!(result.is_ok(), "Failed to parse OHLC: {:?}", result.err());

    let ohlc = result.unwrap();

    // OHLC should have at least one entry
    assert!(!ohlc.is_empty(), "OHLC should not be empty");

    // Verify structure
    for (symbol, data) in &ohlc {
        assert!(!symbol.is_empty());
        assert!(data.instrument_token > 0);
        assert!(data.ohlc.high >= data.ohlc.low);
    }
}

#[wasm_bindgen_test]
fn test_position_pnl_fields() {
    let result: Result<Positions, _> = extract_data(POSITIONS_JSON);
    let positions = result.unwrap();

    // Find a position with non-zero PnL
    let with_pnl = positions.net.iter().find(|p| p.pnl != 0.0);
    if let Some(pos) = with_pnl {
        // Verify PnL fields are populated
        assert!(pos.m2m != 0.0 || pos.pnl != 0.0);
    }
}

#[wasm_bindgen_test]
fn test_holding_day_change() {
    let result: Result<Holdings, _> = extract_data(HOLDINGS_JSON);
    let holdings = result.unwrap();

    // Verify day change calculations exist
    for holding in &holdings {
        // Day change can be positive, negative, or zero
        // Just verify the fields are parsed
        let _ = holding.day_change;
        let _ = holding.day_change_percentage;
    }
}

// ============================================================================
// API Client Type Compilation Tests
// ============================================================================
// These tests verify that API methods compile correctly for WASM target.
// They don't make actual HTTP calls, just ensure type compatibility.

#[wasm_bindgen_test]
fn test_api_methods_compile() {
    // This test verifies that KiteConnect methods are available in WASM
    // by checking that we can reference them (without calling)
    let kite = KiteConnect::builder("test_key")
        .access_token("test_token")
        .build()
        .unwrap();

    // Verify method signatures compile (async methods return futures)
    let _ = kite.get_login_url();

    // Note: We can't actually call these methods without a network,
    // but verifying they compile is the goal of this test
}

#[wasm_bindgen_test]
fn test_order_params_serialization() {
    let params = OrderParams {
        exchange: Some("NSE".to_string()),
        tradingsymbol: Some("INFY".to_string()),
        transaction_type: Some("BUY".to_string()),
        quantity: Some(10),
        price: Some(1500.0),
        product: Some("CNC".to_string()),
        order_type: Some("LIMIT".to_string()),
        validity: Some("DAY".to_string()),
        validity_ttl: None,
        disclosed_quantity: None,
        trigger_price: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: Some("wasm_test".to_string()),
    };

    // Verify serialization works
    let json = serde_json::to_string(&params);
    assert!(json.is_ok());

    // Verify deserialization works
    let parsed: Result<OrderParams, _> = serde_json::from_str(&json.unwrap());
    assert!(parsed.is_ok());
}

#[wasm_bindgen_test]
fn test_convert_position_params_serialization() {
    let params = ConvertPositionParams {
        exchange: "NSE".to_string(),
        tradingsymbol: "INFY".to_string(),
        old_product: "MIS".to_string(),
        new_product: "CNC".to_string(),
        position_type: "day".to_string(),
        transaction_type: "BUY".to_string(),
        quantity: 10,
    };

    let json = serde_json::to_string(&params);
    assert!(json.is_ok());
}

#[wasm_bindgen_test]
fn test_json_roundtrip_position() {
    use kiteconnect_rs::Position;

    // Test that Position can be serialized and deserialized
    let json = r#"{
        "tradingsymbol": "TEST",
        "exchange": "NSE",
        "instrument_token": 12345,
        "product": "CNC",
        "quantity": 10,
        "overnight_quantity": 5,
        "multiplier": 1.0,
        "average_price": 100.0,
        "close_price": 99.0,
        "last_price": 101.0,
        "value": 1010.0,
        "pnl": 10.0,
        "m2m": 20.0,
        "unrealised": 10.0,
        "realised": 0.0,
        "buy_quantity": 10,
        "buy_price": 100.0,
        "buy_value": 1000.0,
        "buy_m2m": 1000.0,
        "sell_quantity": 0,
        "sell_price": 0.0,
        "sell_value": 0.0,
        "sell_m2m": 0.0,
        "day_buy_quantity": 10,
        "day_buy_price": 100.0,
        "day_buy_value": 1000.0,
        "day_sell_quantity": 0,
        "day_sell_price": 0.0,
        "day_sell_value": 0.0
    }"#;

    let parsed: Result<Position, _> = serde_json::from_str(json);
    assert!(parsed.is_ok());

    let pos = parsed.unwrap();
    assert_eq!(pos.tradingsymbol, "TEST");
    assert_eq!(pos.quantity, 10);
    assert_eq!(pos.pnl, 10.0);
}

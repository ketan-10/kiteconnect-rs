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

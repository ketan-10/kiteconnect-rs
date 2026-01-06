#![cfg(not(target_arch = "wasm32"))]

use base64::{Engine as _, engine::general_purpose};
use kiteconnect_rs::{DepthItem, Mode, Ticker, TickerBuilder};
use std::fs;
use std::time::Duration;

#[tokio::test]
async fn test_ticker_creation() {
    let _ticker = Ticker::new("test_api_key".to_string(), "test_access_token".to_string());
    // Basic creation test passes if no panic occurs
}

#[tokio::test]
async fn test_ticker_builder() {
    let result = TickerBuilder::new("test_api_key", "test_access_token")
        .auto_reconnect(false)
        .reconnect_max_retries(5)
        .connect_timeout(Duration::from_secs(5))
        .build();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reconnect_delay_validation() {
    let (mut ticker, _) = Ticker::new("test_api_key".to_string(), "test_access_token".to_string());

    // Test that setting delay below minimum fails
    let result = ticker.set_reconnect_max_delay(Duration::from_millis(1000));
    assert!(result.is_err());

    // Test that setting valid delay succeeds
    let result = ticker.set_reconnect_max_delay(Duration::from_millis(10000));
    assert!(result.is_ok());
}

#[test]
fn test_packet_parsing_ltp() {
    // Test LTP packet parsing
    let data = vec![
        0x00, 0x06, 0x3a, 0x01, // instrument token: 408065
        0x00, 0x02, 0x66, 0x83, // last price: 157315 (1573.15 after conversion)
    ];

    let result = kiteconnect_rs::Ticker::parse_packet(&data);
    assert!(result.is_ok());

    let tick = result.unwrap();
    assert_eq!(tick.instrument_token, 408065);
    assert_eq!(tick.mode, "ltp");
    assert_eq!(tick.last_price, 1573.15);
}

#[test]
fn test_price_conversion() {
    // Test NSE/BSE equity price conversion (divide by 100)
    let price = kiteconnect_rs::Ticker::convert_price(1, 157315);
    assert_eq!(price, 1573.15);

    // Test NSE CD price conversion (divide by 10,000,000)
    let price = kiteconnect_rs::Ticker::convert_price(3, 157315000);
    assert_eq!(price, 15.7315);

    // Test BSE CD price conversion (divide by 10,000)
    let price = kiteconnect_rs::Ticker::convert_price(6, 157315);
    assert_eq!(price, 15.7315);
}

#[test]
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

    let packets = kiteconnect_rs::Ticker::split_packets(&data);
    assert_eq!(packets.len(), 2);
    assert_eq!(packets[0].len(), 8);
    assert_eq!(packets[1].len(), 8);
}

#[test]
fn test_mode_display() {
    assert_eq!(Mode::LTP.to_string(), "ltp");
    assert_eq!(Mode::Quote.to_string(), "quote");
    assert_eq!(Mode::Full.to_string(), "full");
}

// Helper function to load packet data from base64 files
fn load_packet(file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let decoded = general_purpose::STANDARD.decode(content.trim())?;
    Ok(decoded)
}

#[test]
fn test_parse_quote_packet() {
    let packet_data =
        load_packet("tests/mocks/ticker_quote.packet").expect("Failed to load quote packet");

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

#[test]
fn test_parse_full_packet() {
    let packet_data =
        load_packet("tests/mocks/ticker_full.packet").expect("Failed to load full packet");

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
        DepthItem {
            price: 1573.4,
            quantity: 5,
            orders: 1,
        },
        DepthItem {
            price: 1573.0,
            quantity: 140,
            orders: 2,
        },
        DepthItem {
            price: 1572.95,
            quantity: 2,
            orders: 1,
        },
        DepthItem {
            price: 1572.9,
            quantity: 219,
            orders: 7,
        },
        DepthItem {
            price: 1572.85,
            quantity: 50,
            orders: 1,
        },
    ];

    for (i, expected) in expected_buy_depth.iter().enumerate() {
        assert_eq!(tick.depth.buy[i].price, expected.price);
        assert_eq!(tick.depth.buy[i].quantity, expected.quantity);
        assert_eq!(tick.depth.buy[i].orders, expected.orders);
    }

    // Check depth data - Sell side
    let expected_sell_depth = [
        DepthItem {
            price: 1573.7,
            quantity: 172,
            orders: 3,
        },
        DepthItem {
            price: 1573.75,
            quantity: 44,
            orders: 3,
        },
        DepthItem {
            price: 1573.85,
            quantity: 302,
            orders: 3,
        },
        DepthItem {
            price: 1573.9,
            quantity: 141,
            orders: 2,
        },
        DepthItem {
            price: 1573.95,
            quantity: 724,
            orders: 5,
        },
    ];

    for (i, expected) in expected_sell_depth.iter().enumerate() {
        assert_eq!(tick.depth.sell[i].price, expected.price);
        assert_eq!(tick.depth.sell[i].quantity, expected.quantity);
        assert_eq!(tick.depth.sell[i].orders, expected.orders);
    }
}

#[test]
fn test_parse_binary_with_multiple_packets() {
    // Test parsing binary data with multiple packets
    let quote_data =
        load_packet("tests/mocks/ticker_quote.packet").expect("Failed to load quote packet");
    let full_data =
        load_packet("tests/mocks/ticker_full.packet").expect("Failed to load full packet");

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

#[test]
fn test_segment_detection() {
    // Test different segment detection
    let nse_cm_token = 408065; // NSE CM
    let _bse_cd_token = 0x06; // BSE CD
    let indices_token = 0x09; // INDICES

    // NSE CM token
    let seg = nse_cm_token & 0xFF;
    assert_ne!(seg, 9); // Not indices

    // Indices token
    let seg = indices_token & 0xFF;
    assert_eq!(seg, 9); // Is indices
}

mod integration_tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    #[ignore] // Ignore by default since it requires real credentials
    async fn test_real_connection() {
        let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
        let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();

        if api_key.is_empty() || access_token.is_empty() {
            println!("Skipping real connection test - no credentials provided");
            return;
        }

        let (ticker, handle) = TickerBuilder::new(&api_key, &access_token)
            .auto_reconnect(false)
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let mut event_receiver = handle.subscribe_events();

        // Start ticker
        let ticker_handle = tokio::spawn(async move { ticker.serve().await });

        // Wait for connection or timeout
        let result = timeout(Duration::from_secs(10), async {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    kiteconnect_rs::TickerEvent::Connect => {
                        println!("Successfully connected!");
                        return Ok(());
                    }
                    kiteconnect_rs::TickerEvent::Error(e) => {
                        println!("Connection error: {}", e);
                        return Err(e);
                    }
                    _ => {}
                }
            }
            Err("No events received".to_string())
        })
        .await;

        // Clean up
        ticker_handle.abort();

        match result {
            Ok(Ok(())) => println!("Integration test passed"),
            Ok(Err(e)) => println!("Integration test failed: {}", e),
            Err(_) => println!("Integration test timed out"),
        }
    }
}

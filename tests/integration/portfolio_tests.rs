use kiteconnect_rs::{
    KiteConnect,
    portfolio::{ConvertPositionParams, HoldingAuthParams, HoldingsAuthInstruments},
};
use std::time::Duration;

use super::mock_server::KiteMockServer;

#[tokio::test]
async fn test_get_positions() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token for authentication
    kite.set_access_token("test_access_token");

    // Test get_positions
    let positions = kite.get_positions().await;

    assert!(
        positions.is_ok(),
        "Failed to get positions: {:?}",
        positions.err()
    );

    let positions_data = positions.unwrap();

    // Check that we have both day and net positions
    assert!(
        !positions_data.day.is_empty(),
        "Day positions should not be empty"
    );
    assert!(
        !positions_data.net.is_empty(),
        "Net positions should not be empty"
    );

    // Verify some fields in day positions
    for position in &positions_data.day {
        assert!(
            !position.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
    }

    // Verify some fields in net positions
    for position in &positions_data.net {
        assert!(
            !position.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
    }

    // Check specific values from mock data
    assert_eq!(positions_data.net[0].tradingsymbol, "LEADMINI17DECFUT");
    assert_eq!(positions_data.net[0].exchange, "MCX");
    assert_eq!(positions_data.net[0].instrument_token, 53496327);
}

#[tokio::test]
async fn test_get_holdings() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token for authentication
    kite.set_access_token("test_access_token");

    // Test get_holdings
    let holdings = kite.get_holdings().await;

    assert!(
        holdings.is_ok(),
        "Failed to get holdings: {:?}",
        holdings.err()
    );

    let holdings_data = holdings.unwrap();
    assert!(!holdings_data.is_empty(), "Holdings should not be empty");

    // Verify trading symbols are present
    for holding in &holdings_data {
        assert!(
            !holding.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
    }

    // Check specific values from mock data including MTF fields
    assert_eq!(holdings_data[0].tradingsymbol, "AARON");
    assert_eq!(holdings_data[0].exchange, "NSE");
    assert_eq!(holdings_data[0].instrument_token, 263681);
    assert_eq!(holdings_data[0].isin, "INE721Z01010");

    // Test MTF fields
    assert_eq!(holdings_data[0].mtf.quantity, 1000);
    assert_eq!(holdings_data[0].mtf.value, 100000.0);
    assert_eq!(holdings_data[0].mtf.average_price, 100.0);
}

#[tokio::test]
async fn test_get_auction_instruments() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token for authentication
    kite.set_access_token("test_access_token");

    // Test get_auction_instruments
    let auction_instruments = kite.get_auction_instruments().await;

    assert!(
        auction_instruments.is_ok(),
        "Failed to get auction instruments: {:?}",
        auction_instruments.err()
    );

    let instruments = auction_instruments.unwrap();
    assert!(
        !instruments.is_empty(),
        "Auction instruments should not be empty"
    );

    // Verify required fields
    for instrument in &instruments {
        assert!(
            !instrument.auction_number.is_empty(),
            "Auction number should not be empty"
        );
        assert!(instrument.quantity > 0, "Quantity should be greater than 0");
        assert!(
            !instrument.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
    }

    // Check specific values from mock data
    assert_eq!(instruments[0].tradingsymbol, "ASHOKLEY");
    assert_eq!(instruments[0].exchange, "NSE");
    assert_eq!(instruments[0].auction_number, "20");
    assert_eq!(instruments[0].quantity, 1);
}

#[tokio::test]
async fn test_convert_position() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token for authentication
    kite.set_access_token("test_access_token");

    // Create position conversion parameters
    let params = ConvertPositionParams {
        exchange: "NSE".to_string(),
        tradingsymbol: "SBIN".to_string(),
        old_product: "MIS".to_string(),
        new_product: "CNC".to_string(),
        position_type: "day".to_string(),
        transaction_type: "BUY".to_string(),
        quantity: 1,
    };

    // Test convert_position
    let result = kite.convert_position(params).await;

    assert!(
        result.is_ok(),
        "Failed to convert position: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        true,
        "Position conversion should return true"
    );
}

#[tokio::test]
async fn test_initiate_holdings_auth() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token for authentication
    kite.set_access_token("test_access_token");

    // Create holdings authorization parameters
    let params = HoldingAuthParams {
        auth_type: "equity".to_string(),
        transfer_type: "pre".to_string(),
        exec_date: "2024-01-01".to_string(),
        instruments: Some(vec![
            HoldingsAuthInstruments {
                isin: "INE002A01018".to_string(),
                quantity: 50.0,
            },
            HoldingsAuthInstruments {
                isin: "INE009A01021".to_string(),
                quantity: 50.0,
            },
        ]),
    };

    // Test initiate_holdings_auth
    let result = kite.initiate_holdings_auth(params).await;

    assert!(
        result.is_ok(),
        "Failed to initiate holdings auth: {:?}",
        result.err()
    );

    let auth_response = result.unwrap();
    assert_eq!(auth_response.request_id, "na8QgCeQm05UHG6NL9sAGRzdfSF64UdB");
    assert!(
        auth_response.redirect_url.is_some(),
        "Redirect URL should be present"
    );
    let redirect_url = auth_response.redirect_url.unwrap();
    assert!(
        redirect_url.contains("https://kite.zerodha.com"),
        "Redirect URL should contain kite base URL"
    );
    assert!(
        redirect_url.contains("test_api_key"),
        "Redirect URL should contain API key"
    );
    assert!(
        redirect_url.contains("na8QgCeQm05UHG6NL9sAGRzdfSF64UdB"),
        "Redirect URL should contain request ID"
    );
}

#[tokio::test]
async fn test_portfolio_error_handling() {
    // Create KiteConnect client with invalid base URL to trigger errors
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url("http://invalid-url-that-does-not-exist.com")
        .timeout(Duration::from_secs(1))
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_token");

    // Test that portfolio-specific errors are properly handled
    let positions = kite.get_positions().await;
    assert!(positions.is_err(), "Expected error for invalid URL");

    let holdings = kite.get_holdings().await;
    assert!(holdings.is_err(), "Expected error for invalid URL");

    let auctions = kite.get_auction_instruments().await;
    assert!(auctions.is_err(), "Expected error for invalid URL");
}

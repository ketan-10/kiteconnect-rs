use kiteconnect_rs::{KiteConnect, orders::OrderParams};
use std::time::Duration;

use super::mock_server::KiteMockServer;

#[tokio::test]
async fn test_get_orders() {
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

    // Test get_orders
    let orders = kite.get_orders().await;

    assert!(orders.is_ok(), "Failed to get orders: {:?}", orders.err());

    let orders_data = orders.unwrap();
    assert!(!orders_data.is_empty(), "Orders should not be empty");

    // Verify first order details from mock data
    let first_order = &orders_data[0];
    assert_eq!(first_order.order_id, "100000000000000");
    assert_eq!(first_order.placed_by, "XXXXXX");
    assert_eq!(first_order.status, "CANCELLED");
    assert_eq!(first_order.exchange, "CDS");
    assert_eq!(first_order.tradingsymbol, "USDINR21JUNFUT");
    assert_eq!(first_order.instrument_token, 412675);
    assert_eq!(first_order.order_type, "LIMIT");
    assert_eq!(first_order.transaction_type, "BUY");
    assert_eq!(first_order.validity, "DAY");
    assert_eq!(first_order.product, "NRML");
    assert_eq!(first_order.quantity, 1.0);
    assert_eq!(first_order.price, 72.0);
    assert_eq!(first_order.cancelled_quantity, 1.0);

    // Verify second order (completed order)
    let second_order = &orders_data[1];
    assert_eq!(second_order.order_id, "300000000000000");
    assert_eq!(second_order.status, "COMPLETE");
    assert_eq!(second_order.exchange, "NSE");
    assert_eq!(second_order.tradingsymbol, "IOC");
    assert_eq!(second_order.filled_quantity, 1.0);
    assert_eq!(second_order.average_price, 109.4);
}

#[tokio::test]
async fn test_get_trades() {
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

    // Test get_trades
    let trades = kite.get_trades().await;

    assert!(trades.is_ok(), "Failed to get trades: {:?}", trades.err());

    let trades_data = trades.unwrap();
    assert!(!trades_data.is_empty(), "Trades should not be empty");

    // Verify first trade details from mock data
    let first_trade = &trades_data[0];
    assert_eq!(first_trade.trade_id, "10000000");
    assert_eq!(first_trade.order_id, "200000000000000");
    assert_eq!(first_trade.exchange, "NSE");
    assert_eq!(first_trade.tradingsymbol, "SBIN");
    assert_eq!(first_trade.instrument_token, 779521);
    assert_eq!(first_trade.product, "CNC");
    assert_eq!(first_trade.average_price, 420.65);
    assert_eq!(first_trade.quantity, 1.0);
    assert_eq!(first_trade.transaction_type, "BUY");

    // Verify second trade
    let second_trade = &trades_data[1];
    assert_eq!(second_trade.trade_id, "40000000");
    assert_eq!(second_trade.exchange, "CDS");
    assert_eq!(second_trade.tradingsymbol, "USDINR21JUNFUT");
    assert_eq!(second_trade.average_price, 72.755);
}

#[tokio::test]
async fn test_get_order_history() {
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

    // Test get_order_history
    let order_history = kite.get_order_history("151220000000000").await;

    assert!(
        order_history.is_ok(),
        "Failed to get order history: {:?}",
        order_history.err()
    );

    let history_data = order_history.unwrap();
    assert!(
        !history_data.is_empty(),
        "Order history should not be empty"
    );

    // Verify order history details
    for order in &history_data {
        assert!(!order.order_id.is_empty(), "Order ID should not be empty");
        assert!(
            !order.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
    }
}

#[tokio::test]
async fn test_get_order_trades() {
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

    // Test get_order_trades
    let order_trades = kite.get_order_trades("151220000000000").await;

    assert!(
        order_trades.is_ok(),
        "Failed to get order trades: {:?}",
        order_trades.err()
    );

    let trades_data = order_trades.unwrap();
    assert!(!trades_data.is_empty(), "Order trades should not be empty");

    // Verify order trades details
    for trade in &trades_data {
        assert!(!trade.trade_id.is_empty(), "Trade ID should not be empty");
        assert!(!trade.order_id.is_empty(), "Order ID should not be empty");
        assert!(
            !trade.tradingsymbol.is_empty(),
            "Trading symbol should not be empty"
        );
        assert!(trade.quantity > 0.0, "Quantity should be greater than 0");
    }
}

#[tokio::test]
async fn test_place_order() {
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

    // Create order parameters
    let order_params = OrderParams {
        exchange: Some("NSE".to_string()),
        tradingsymbol: Some("SBIN".to_string()),
        transaction_type: Some("BUY".to_string()),
        order_type: Some("LIMIT".to_string()),
        quantity: Some(1),
        price: Some(420.0),
        product: Some("CNC".to_string()),
        validity: Some("DAY".to_string()),
        disclosed_quantity: None,
        trigger_price: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: None,
        validity_ttl: None,
    };

    // Test place_order
    let result = kite.place_order("regular", order_params).await;

    assert!(result.is_ok(), "Failed to place order: {:?}", result.err());

    let order_response = result.unwrap();
    assert_eq!(
        order_response.order_id, "151220000000000",
        "Order ID should match mock response"
    );
}

#[tokio::test]
async fn test_modify_order() {
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

    // Create order modification parameters
    let order_params = OrderParams {
        price: Some(425.0), // Modified price
        quantity: Some(2),  // Modified quantity
        order_type: Some("LIMIT".to_string()),
        validity: Some("DAY".to_string()),
        exchange: None,
        tradingsymbol: None,
        transaction_type: None,
        product: None,
        disclosed_quantity: None,
        trigger_price: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: None,
        validity_ttl: None,
    };

    // Test modify_order
    let result = kite
        .modify_order("regular", "151220000000000", order_params)
        .await;

    assert!(result.is_ok(), "Failed to modify order: {:?}", result.err());

    let order_response = result.unwrap();
    assert!(
        !order_response.order_id.is_empty(),
        "Order ID should not be empty"
    );
}

#[tokio::test]
async fn test_cancel_order() {
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

    // Test cancel_order without parent order ID
    let result = kite.cancel_order("regular", "151220000000000", None).await;

    assert!(result.is_ok(), "Failed to cancel order: {:?}", result.err());

    let order_response = result.unwrap();
    assert_eq!(
        order_response.order_id, "151220000000000",
        "Order ID should match"
    );
}

#[tokio::test]
async fn test_cancel_order_with_parent() {
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

    // Test cancel_order with parent order ID
    let result = kite
        .cancel_order("regular", "151220000000000", Some("parent_order_123"))
        .await;

    assert!(
        result.is_ok(),
        "Failed to cancel order with parent: {:?}",
        result.err()
    );

    let order_response = result.unwrap();
    assert_eq!(
        order_response.order_id, "151220000000000",
        "Order ID should match"
    );
}

#[tokio::test]
async fn test_exit_order() {
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

    // Test exit_order (alias for cancel_order)
    let result = kite.exit_order("regular", "151220000000000", None).await;

    assert!(result.is_ok(), "Failed to exit order: {:?}", result.err());

    let order_response = result.unwrap();
    assert_eq!(
        order_response.order_id, "151220000000000",
        "Order ID should match"
    );
}

#[tokio::test]
async fn test_order_error_handling() {
    // Create KiteConnect client with invalid base URL to trigger errors
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url("http://invalid-url-that-does-not-exist.com")
        .timeout(Duration::from_secs(1))
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_token");

    // Test that order-specific errors are properly handled
    let orders = kite.get_orders().await;
    assert!(orders.is_err(), "Expected error for invalid URL");

    let trades = kite.get_trades().await;
    assert!(trades.is_err(), "Expected error for invalid URL");

    let order_history = kite.get_order_history("123").await;
    assert!(order_history.is_err(), "Expected error for invalid URL");

    let order_trades = kite.get_order_trades("123").await;
    assert!(order_trades.is_err(), "Expected error for invalid URL");

    // Test place order with empty params
    let empty_params = OrderParams {
        exchange: None,
        tradingsymbol: None,
        transaction_type: None,
        order_type: None,
        quantity: None,
        price: None,
        product: None,
        validity: None,
        disclosed_quantity: None,
        trigger_price: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: None,
        validity_ttl: None,
    };

    let place_result = kite.place_order("regular", empty_params).await;
    assert!(place_result.is_err(), "Expected error for invalid URL");
}

use crate::integration::mock_server::KiteMockServer;
use kiteconnect_rs::*;

#[tokio::test]
async fn test_get_order_margins() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnectBuilder::new("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_access_token");

    let params = OrderMarginParam {
        exchange: "NSE".to_string(),
        trading_symbol: "INFY".to_string(),
        transaction_type: "BUY".to_string(),
        variety: "regular".to_string(),
        product: "CNC".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        price: None,
        trigger_price: None,
    };

    // Test compact order margins
    let compact_result = kite
        .get_order_margins(GetMarginParams {
            order_params: vec![params.clone()],
            compact: true,
        })
        .await;

    assert!(compact_result.is_ok());
    let compact_margins = compact_result.unwrap();
    assert_eq!(compact_margins.len(), 1);
    assert_eq!(compact_margins[0].trading_symbol, "INFY");
    assert!(compact_margins[0].total > 0.0);

    // Test detailed order margins
    let detailed_result = kite
        .get_order_margins(GetMarginParams {
            order_params: vec![params],
            compact: false,
        })
        .await;

    assert!(detailed_result.is_ok());
    let detailed_margins = detailed_result.unwrap();
    assert_eq!(detailed_margins.len(), 1);
    assert!(detailed_margins[0].charges.transaction_tax >= 0.0);
    assert!(detailed_margins[0].charges.stamp_duty >= 0.0);
    assert!(detailed_margins[0].charges.gst.total >= 0.0);
    assert!(detailed_margins[0].charges.total >= 0.0);
    assert!(detailed_margins[0].total > 0.0);
}

#[tokio::test]
async fn test_get_basket_margins() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnectBuilder::new("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_access_token");

    let params = OrderMarginParam {
        exchange: "NSE".to_string(),
        trading_symbol: "INFY".to_string(),
        transaction_type: "BUY".to_string(),
        variety: "regular".to_string(),
        product: "CNC".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        price: None,
        trigger_price: None,
    };

    let result = kite
        .get_basket_margins(GetBasketParams {
            order_params: vec![params],
            compact: true,
            consider_positions: true,
        })
        .await;

    assert!(result.is_ok());
    let basket_margins = result.unwrap();
    assert_eq!(basket_margins.orders.len(), 2);
}

#[tokio::test]
async fn test_get_order_charges() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnectBuilder::new("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_access_token");

    let params = vec![
        OrderChargesParam {
            order_id: "11111".to_string(),
            exchange: "NSE".to_string(),
            trading_symbol: "INFY".to_string(),
            transaction_type: "BUY".to_string(),
            variety: "regular".to_string(),
            product: "CNC".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            average_price: 560.0,
        },
        OrderChargesParam {
            order_id: "22222".to_string(),
            exchange: "MCX".to_string(),
            trading_symbol: "GOLDPETAL23JULFUT".to_string(),
            transaction_type: "SELL".to_string(),
            variety: "regular".to_string(),
            product: "NRML".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            average_price: 5862.0,
        },
        OrderChargesParam {
            order_id: "33333".to_string(),
            exchange: "NFO".to_string(),
            trading_symbol: "NIFTY2371317900PE".to_string(),
            transaction_type: "BUY".to_string(),
            variety: "regular".to_string(),
            product: "NRML".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 100.0,
            average_price: 1.5,
        },
    ];

    let result = kite
        .get_order_charges(GetChargesParams {
            order_params: params,
        })
        .await;

    assert!(result.is_ok());
    let order_charges = result.unwrap();
    assert_eq!(order_charges.len(), 3);
}

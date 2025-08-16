use crate::integration::mock_server::KiteMockServer;
use kiteconnect_rs::KiteConnect;

#[tokio::test]
async fn test_get_quote() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let instruments = vec!["NSE:INFY"];
    let result = kite.get_quote(&instruments).await;

    assert!(result.is_ok());
    let quote = result.unwrap();

    assert!(quote.contains_key("NSE:INFY"));

    if let Some(infy_quote) = quote.get("NSE:INFY") {
        assert_eq!(infy_quote.instrument_token, 408065);
        assert_eq!(infy_quote.last_price, 1412.95);
    } else {
        panic!("NSE:INFY quote not found");
    }
}

#[tokio::test]
async fn test_get_ltp() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let instruments = vec!["NSE:INFY"];
    let result = kite.get_ltp(&instruments).await;

    assert!(result.is_ok());
    let ltp = result.unwrap();

    assert!(ltp.contains_key("NSE:INFY"));

    if let Some(infy_ltp) = ltp.get("NSE:INFY") {
        assert_eq!(infy_ltp.instrument_token, 408065);
        assert_eq!(infy_ltp.last_price, 1074.35);
    } else {
        panic!("NSE:INFY LTP not found");
    }
}

#[tokio::test]
async fn test_get_ohlc() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let instruments = vec!["NSE:INFY"];
    let result = kite.get_ohlc(&instruments).await;

    assert!(result.is_ok());
    let ohlc = result.unwrap();

    assert!(ohlc.contains_key("NSE:INFY"));

    if let Some(infy_ohlc) = ohlc.get("NSE:INFY") {
        assert_eq!(infy_ohlc.instrument_token, 408065);
        assert_eq!(infy_ohlc.last_price, 1075.0);
        assert_eq!(infy_ohlc.ohlc.open, 1085.8);
        assert_eq!(infy_ohlc.ohlc.high, 1085.9);
        assert_eq!(infy_ohlc.ohlc.low, 1070.9);
        assert_eq!(infy_ohlc.ohlc.close, 1075.8);
    } else {
        panic!("NSE:INFY OHLC not found");
    }
}

#[tokio::test]
async fn test_get_historical_data() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let result = kite
        .get_historical_data(
            123,
            "myinterval",
            "2017-12-15 09:15:00",
            "2017-12-15 15:30:00",
            true,
            false,
        )
        .await;

    if let Err(ref e) = result {
        eprintln!("Historical data error: {:?}", e);
    }
    assert!(result.is_ok());
    let historical_data = result.unwrap();

    // Verify data is sorted by date
    for i in 0..historical_data.len() - 1 {
        let current_time = historical_data[i].date.as_datetime().unwrap();
        let next_time = historical_data[i + 1].date.as_datetime().unwrap();
        assert!(
            current_time <= next_time,
            "Historical data should be sorted by date"
        );
    }

    // Verify we have some data
    assert!(!historical_data.is_empty());

    // Verify first candle
    let first_candle = &historical_data[0];
    assert_eq!(first_candle.open, 1704.5);
    assert_eq!(first_candle.high, 1705.0);
    assert_eq!(first_candle.low, 1699.25);
    assert_eq!(first_candle.close, 1702.8);
}

#[tokio::test]
async fn test_get_historical_data_with_oi() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let result = kite
        .get_historical_data(
            456,
            "myinterval",
            "2017-12-15 09:15:00",
            "2017-12-15 15:30:00",
            true,
            true,
        )
        .await;

    if let Err(ref e) = result {
        eprintln!("Historical data with OI error: {:?}", e);
    }
    assert!(result.is_ok());
    let historical_data = result.unwrap();

    // Verify data is sorted by date
    for i in 0..historical_data.len() - 1 {
        let current_time = historical_data[i].date.as_datetime().unwrap();
        let next_time = historical_data[i + 1].date.as_datetime().unwrap();
        assert!(
            current_time <= next_time,
            "Historical data should be sorted by date"
        );
    }

    // Verify we have some data
    assert_eq!(historical_data.len(), 6);

    // Verify OI data is present
    for candle in &historical_data {
        assert_ne!(candle.oi, 0, "OI should be present when requested");
    }
}

#[tokio::test]
async fn test_get_instruments() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let result = kite.get_instruments().await;

    assert!(result.is_ok());
    let instruments = result.unwrap();

    // Verify we have some data
    assert!(!instruments.is_empty());

    // Check for a specific instrument
    let adaniports = instruments.iter().find(|i| i.instrument_token == 3861249);
    assert!(adaniports.is_some());

    if let Some(instrument) = adaniports {
        assert_eq!(instrument.tradingsymbol, "ADANIPORTS");
        assert_eq!(instrument.exchange, "NSE");
        assert_eq!(instrument.instrument_type, "EQ");
    }

    // Test an instrument with expiry
    let banknifty_option = instruments.iter().find(|i| i.instrument_token == 12073986);
    assert!(banknifty_option.is_some());

    if let Some(instrument) = banknifty_option {
        assert_eq!(instrument.strike, 23500.0);
        assert_eq!(instrument.instrument_type, "CE");
        // Verify expiry is correctly parsed (should be 2018-01-24 after timezone conversion)
        if let Some(expiry_dt) = instrument.expiry.as_datetime() {
            assert_eq!(expiry_dt.format("%Y-%m-%d").to_string(), "2018-01-24");
        }
    }
}

#[tokio::test]
async fn test_get_instruments_by_exchange() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let result = kite.get_instruments_by_exchange("nse").await;

    assert!(result.is_ok());
    let instruments = result.unwrap();

    // Verify all instruments are from NSE exchange
    for instrument in &instruments {
        assert_eq!(instrument.exchange, "NSE");
    }

    // Verify we have some data
    assert!(!instruments.is_empty());
}

#[tokio::test]
async fn test_get_mf_instruments() {
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .build()
        .expect("Failed to create KiteConnect instance");

    kite.set_access_token("test_access_token");

    let result = kite.get_mf_instruments().await;

    assert!(result.is_ok());
    let instruments = result.unwrap();

    // Verify we have some data
    assert!(!instruments.is_empty());

    // Verify all instruments have non-empty trading symbols
    for instrument in &instruments {
        assert!(!instrument.tradingsymbol.is_empty());
    }
}

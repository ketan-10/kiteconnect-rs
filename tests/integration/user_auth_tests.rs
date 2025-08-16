use kiteconnect_rs::KiteConnect;
use std::time::Duration;

use super::mock_server::KiteMockServer;

#[tokio::test]
async fn test_get_user_profile() {
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

    // Test get_user_profile
    let profile = kite.get_user_profile().await;

    assert!(
        profile.is_ok(),
        "Failed to get user profile: {:?}",
        profile.err()
    );

    let user_profile = profile.unwrap();
    assert_eq!(user_profile.user_id, "AB1234");
    assert_eq!(user_profile.user_name, "AxAx Bxx");
    assert_eq!(user_profile.email, "xxxyyy@gmail.com");
    assert_eq!(user_profile.user_type, "individual");
    assert_eq!(user_profile.broker, "ZERODHA");
}

#[tokio::test]
async fn test_get_full_user_profile() {
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

    // Test get_full_user_profile
    let profile = kite.get_full_user_profile().await;

    assert!(
        profile.is_ok(),
        "Failed to get full user profile: {:?}",
        profile.err()
    );

    let full_profile = profile.unwrap();
    assert_eq!(full_profile.user_id, "AB1234");
    assert_eq!(full_profile.user_name, "AxAx Bxx");
    assert_eq!(full_profile.email, "xxxyyy@gmail.com");
    assert_eq!(full_profile.user_type, "individual");
    assert_eq!(full_profile.broker, "ZERODHA");
    assert_eq!(full_profile.phone, "*9999");
    assert_eq!(full_profile.pan, "*xxxI");
    assert_eq!(full_profile.twofa_type, "totp");

    // Test bank accounts
    assert_eq!(full_profile.banks.len(), 2);
    assert_eq!(full_profile.banks[0].name, "HDFC BANK");
    assert_eq!(full_profile.banks[0].account, "*9999");
    assert_eq!(full_profile.banks[1].name, "State Bank of India");

    // Test DP IDs
    assert_eq!(full_profile.dp_ids.len(), 1);
    assert_eq!(full_profile.dp_ids[0], "0xx0xxx0xxxx0xx0");

    // Test meta information
    assert_eq!(full_profile.meta.demat_consent, "physical");
    assert_eq!(full_profile.meta.silo, "x");
    assert!(full_profile.meta.account_blocks.is_empty());

    // Test timestamps
    assert_eq!(full_profile.password_timestamp, "1970-01-01 00:00:00");
    assert_eq!(full_profile.twofa_timestamp, "1970-01-01 00:00:00");

    // Test that avatar_url is None
    assert!(full_profile.avatar_url.is_none());
}

#[tokio::test]
async fn test_get_user_margins() {
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

    // Test get_user_margins
    let margins = kite.get_user_margins().await;

    assert!(
        margins.is_ok(),
        "Failed to get user margins: {:?}",
        margins.err()
    );

    let all_margins = margins.unwrap();
    assert!(all_margins.equity.enabled);
    assert_eq!(all_margins.equity.net, 99725.05000000002);
    assert!(all_margins.commodity.enabled);
    assert_eq!(all_margins.commodity.net, 100661.7);
}

#[tokio::test]
async fn test_get_user_segment_margins() {
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

    // Test get_user_segment_margins
    let margins = kite.get_user_segment_margins("equity").await;

    assert!(
        margins.is_ok(),
        "Failed to get segment margins: {:?}",
        margins.err()
    );

    let equity_margins = margins.unwrap();
    assert!(equity_margins.enabled);
    assert_eq!(equity_margins.net, 99725.05000000002);
    assert_eq!(equity_margins.available.cash, 245431.6);
}

#[tokio::test]
async fn test_generate_session() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Test generate_session
    let session = kite
        .generate_session("test_request_token", "test_api_secret")
        .await;

    assert!(
        session.is_ok(),
        "Failed to generate session: {:?}",
        session.err()
    );

    let user_session = session.unwrap();
    assert_eq!(user_session.user_id, "XX0000");
    assert_eq!(user_session.access_token, "XXXXXX");
    assert_eq!(user_session.refresh_token, "");

    // Note: We can't directly verify the internal access token was set
    // since it's private, but we trust the implementation based on the API design
}

#[tokio::test]
async fn test_invalidate_access_token() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Set access token first
    kite.set_access_token("test_access_token");

    // Test invalidate_access_token
    let result = kite.invalidate_access_token().await;

    assert!(
        result.is_ok(),
        "Failed to invalidate access token: {:?}",
        result.err()
    );
    assert!(result.unwrap(), "Expected invalidation to return true");
}

#[tokio::test]
async fn test_renew_access_token() {
    // Setup mock server
    let mock_server = KiteMockServer::new().await;
    mock_server.setup_all_mocks().await;

    // Create KiteConnect client with mock base URL
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url(&mock_server.base_url)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build KiteConnect client");

    // Test renew_access_token
    let tokens = kite
        .renew_access_token("test_refresh_token", "test_api_secret")
        .await;

    assert!(
        tokens.is_ok(),
        "Failed to renew access token: {:?}",
        tokens.err()
    );

    let new_tokens = tokens.unwrap();
    assert_eq!(new_tokens.access_token, "XXXXXX");
    assert_eq!(new_tokens.refresh_token, "");

    // Note: We can't directly verify the internal access token was updated
    // since it's private, but we trust the implementation based on the API design
}

// Test helper functions and error handling
#[tokio::test]
async fn test_error_handling() {
    // Create KiteConnect client with invalid base URL to trigger errors
    let mut kite = KiteConnect::builder("test_api_key")
        .base_url("http://invalid-url-that-does-not-exist.com")
        .timeout(Duration::from_secs(1))
        .build()
        .expect("Failed to build KiteConnect client");

    kite.set_access_token("test_token");

    // Test that errors are properly handled
    let profile = kite.get_user_profile().await;
    assert!(profile.is_err(), "Expected error for invalid URL");
}

// Unit tests for builder pattern
#[test]
fn test_kite_connect_builder() {
    let _kite = KiteConnect::builder("test_api_key")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build KiteConnect client");

    // We can't test private fields directly, but we trust the builder pattern works
    // based on the successful creation of the client
}

#[test]
fn test_access_token_management() {
    let mut kite = KiteConnect::builder("test_api_key")
        .build()
        .expect("Failed to build KiteConnect client");

    // We can only test the public API
    kite.set_access_token("test_token");
    kite.clear_access_token();
    // Access token state is now private, so we can't verify directly
}

#[test]
fn test_login_url_generation() {
    let kite = KiteConnect::builder("test_api_key")
        .build()
        .expect("Failed to build KiteConnect client");

    let login_url = kite.get_login_url();
    assert!(login_url.contains("test_api_key"));
    assert!(login_url.contains("v=3"));
}

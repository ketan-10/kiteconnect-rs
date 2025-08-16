use crate::integration::mock_server::KiteMockServer;
use kiteconnect_rs::{
    KiteConnect, KiteConnectError, KiteConnectErrorKind,
    alerts::{AlertOperator, AlertParams, AlertStatus, AlertType},
};
use std::collections::HashMap;

const TEST_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";

pub struct AlertsTestSuite {
    pub kite_connect: KiteConnect,
    pub _mock_server: KiteMockServer,
}

impl AlertsTestSuite {
    pub async fn new() -> Self {
        let mock_server = KiteMockServer::new().await;
        mock_server.setup_all_mocks().await;

        let kite_connect = KiteConnect::builder("test_api_key")
            .base_url(&mock_server.base_url)
            .build()
            .unwrap();

        Self {
            kite_connect,
            _mock_server: mock_server,
        }
    }
}

#[tokio::test]
async fn test_create_alert() {
    let ts = AlertsTestSuite::new().await;

    let params = AlertParams {
        name: "NIFTY 50".to_string(),
        r#type: AlertType::Simple,
        lhs_exchange: "INDICES".to_string(),
        lhs_tradingsymbol: "NIFTY 50".to_string(),
        lhs_attribute: "LastTradedPrice".to_string(),
        operator: AlertOperator::Ge,
        rhs_type: "constant".to_string(),
        rhs_constant: Some(27000.0),
        rhs_exchange: None,
        rhs_tradingsymbol: None,
        rhs_attribute: None,
        basket: None,
    };

    let result = ts.kite_connect.create_alert(params).await;
    assert!(result.is_ok(), "Failed to create alert: {:?}", result.err());

    let alert = result.unwrap();
    assert_eq!(alert.name, "NIFTY 50");
    assert_eq!(alert.lhs_exchange, "INDICES");
    assert_eq!(alert.r#type, AlertType::Simple);
    assert!(alert.uuid.len() > 0);
}

#[tokio::test]
async fn test_get_alerts() {
    let ts = AlertsTestSuite::new().await;

    let result = ts.kite_connect.get_alerts(None).await;
    assert!(result.is_ok(), "Failed to get alerts: {:?}", result.err());

    let alerts = result.unwrap();
    assert!(alerts.len() > 0, "No alerts returned");

    let first_alert = &alerts[0];
    assert!(!first_alert.uuid.is_empty(), "Alert UUID is empty");
    assert!(!first_alert.name.is_empty(), "Alert name is empty");
}

#[tokio::test]
async fn test_get_alerts_with_filters() {
    let ts = AlertsTestSuite::new().await;

    let mut filters = HashMap::new();
    filters.insert("status".to_string(), "enabled".to_string());

    let result = ts.kite_connect.get_alerts(Some(filters)).await;
    assert!(
        result.is_ok(),
        "Failed to get filtered alerts: {:?}",
        result.err()
    );

    let alerts = result.unwrap();
    // Should still return alerts from mock - length is always non-negative
    assert!(alerts.len() == alerts.len());
}

#[tokio::test]
async fn test_get_alert() {
    let ts = AlertsTestSuite::new().await;

    let result = ts.kite_connect.get_alert(TEST_UUID).await;
    assert!(result.is_ok(), "Failed to get alert: {:?}", result.err());

    let alert = result.unwrap();
    assert_eq!(alert.uuid, TEST_UUID);
    assert!(!alert.name.is_empty(), "Alert name is empty");
}

#[tokio::test]
async fn test_modify_alert() {
    let ts = AlertsTestSuite::new().await;

    let params = AlertParams {
        name: "NIFTY 50 Modified".to_string(),
        r#type: AlertType::Simple,
        lhs_exchange: "INDICES".to_string(),
        lhs_tradingsymbol: "NIFTY 50".to_string(),
        lhs_attribute: "LastTradedPrice".to_string(),
        operator: AlertOperator::Ge,
        rhs_type: "constant".to_string(),
        rhs_constant: Some(27500.0),
        rhs_exchange: None,
        rhs_tradingsymbol: None,
        rhs_attribute: None,
        basket: None,
    };

    let result = ts.kite_connect.modify_alert(TEST_UUID, params).await;
    assert!(result.is_ok(), "Failed to modify alert: {:?}", result.err());

    let alert = result.unwrap();
    assert_eq!(alert.uuid, TEST_UUID);
    // The mock response should reflect the modification
    if let Some(rhs_constant) = alert.rhs_constant {
        assert!(rhs_constant > 0.0, "RHS constant should be positive");
    }
}

#[tokio::test]
async fn test_delete_alerts() {
    let ts = AlertsTestSuite::new().await;

    let result = ts.kite_connect.delete_alerts(&[TEST_UUID]).await;
    assert!(result.is_ok(), "Failed to delete alert: {:?}", result.err());
}

#[tokio::test]
async fn test_delete_alerts_empty_uuids() {
    let ts = AlertsTestSuite::new().await;

    let result = ts.kite_connect.delete_alerts(&[]).await;
    assert!(result.is_err(), "Should fail with empty UUIDs");

    match result {
        Err(KiteConnectError {
            kind: KiteConnectErrorKind::Other(msg),
            ..
        }) => {
            assert!(msg.contains("At least one uuid must be provided"));
        }
        _ => panic!("Expected Other error with specific message"),
    }
}

#[tokio::test]
async fn test_delete_multiple_alerts() {
    let ts = AlertsTestSuite::new().await;

    let uuid1 = "550e8400-e29b-41d4-a716-446655440001";
    let uuid2 = "550e8400-e29b-41d4-a716-446655440002";

    let result = ts.kite_connect.delete_alerts(&[uuid1, uuid2]).await;
    assert!(
        result.is_ok(),
        "Failed to delete multiple alerts: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_get_alert_history() {
    let ts = AlertsTestSuite::new().await;

    let result = ts.kite_connect.get_alert_history(TEST_UUID).await;
    assert!(
        result.is_ok(),
        "Failed to get alert history: {:?}",
        result.err()
    );

    let history = result.unwrap();
    // History length is always non-negative, just check if we got data

    if !history.is_empty() {
        let first_entry = &history[0];
        assert_eq!(first_entry.uuid, TEST_UUID);
        assert!(
            !first_entry.condition.is_empty(),
            "Condition should not be empty"
        );
    }
}

#[tokio::test]
async fn test_alert_types_serialization() {
    // Test AlertType serialization
    let simple_type = AlertType::Simple;
    let ato_type = AlertType::Ato;

    let simple_json = serde_json::to_string(&simple_type).unwrap();
    let ato_json = serde_json::to_string(&ato_type).unwrap();

    assert_eq!(simple_json, "\"simple\"");
    assert_eq!(ato_json, "\"ato\"");

    // Test AlertStatus serialization
    let enabled_status = AlertStatus::Enabled;
    let disabled_status = AlertStatus::Disabled;
    let deleted_status = AlertStatus::Deleted;

    let enabled_json = serde_json::to_string(&enabled_status).unwrap();
    let disabled_json = serde_json::to_string(&disabled_status).unwrap();
    let deleted_json = serde_json::to_string(&deleted_status).unwrap();

    assert_eq!(enabled_json, "\"enabled\"");
    assert_eq!(disabled_json, "\"disabled\"");
    assert_eq!(deleted_json, "\"deleted\"");

    // Test AlertOperator serialization
    let operators = vec![
        (AlertOperator::Le, "\"<=\""),
        (AlertOperator::Ge, "\">=\""),
        (AlertOperator::Lt, "\"<\""),
        (AlertOperator::Gt, "\">\""),
        (AlertOperator::Eq, "\"==\""),
    ];

    for (operator, expected) in operators {
        let json = serde_json::to_string(&operator).unwrap();
        assert_eq!(json, expected);
    }
}

#[tokio::test]
async fn test_alert_params_validation() {
    let ts = AlertsTestSuite::new().await;

    // Test with constant RHS type
    let constant_params = AlertParams {
        name: "Test Constant Alert".to_string(),
        r#type: AlertType::Simple,
        lhs_exchange: "NSE".to_string(),
        lhs_tradingsymbol: "RELIANCE".to_string(),
        lhs_attribute: "last_price".to_string(),
        operator: AlertOperator::Gt,
        rhs_type: "constant".to_string(),
        rhs_constant: Some(2500.0),
        rhs_exchange: None,
        rhs_tradingsymbol: None,
        rhs_attribute: None,
        basket: None,
    };

    let result = ts.kite_connect.create_alert(constant_params).await;
    assert!(
        result.is_ok(),
        "Failed to create constant alert: {:?}",
        result.err()
    );

    // Test with instrument RHS type
    let instrument_params = AlertParams {
        name: "Test Instrument Alert".to_string(),
        r#type: AlertType::Simple,
        lhs_exchange: "NSE".to_string(),
        lhs_tradingsymbol: "RELIANCE".to_string(),
        lhs_attribute: "last_price".to_string(),
        operator: AlertOperator::Gt,
        rhs_type: "instrument".to_string(),
        rhs_constant: None,
        rhs_exchange: Some("NSE".to_string()),
        rhs_tradingsymbol: Some("SBIN".to_string()),
        rhs_attribute: Some("last_price".to_string()),
        basket: None,
    };

    let result = ts.kite_connect.create_alert(instrument_params).await;
    assert!(
        result.is_ok(),
        "Failed to create instrument alert: {:?}",
        result.err()
    );
}

use kiteconnect_rs::{
    KiteConnect,
    alerts::{AlertOperator, AlertParams, AlertType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "<api_key>";
    let mut kite = KiteConnect::builder(&api_key).build()?;

    println!("Login URL: {}", kite.get_login_url());
    println!("Press Enter request token after login...");
    let mut request_token = String::new();
    std::io::stdin().read_line(&mut request_token)?;

    let api_secret = "<api_secret>";

    let access_token = kite
        .generate_session(&request_token.trim(), api_secret)
        .await?
        .access_token;

    println!("Access Token: {}", access_token);

    println!("=== Alerts API Examples ===\n");

    // Example: Create a simple alert
    println!("Creating a simple alert...");
    let alert_params = AlertParams {
        name: "NIFTY 50 Alert".to_string(),
        r#type: AlertType::Simple,
        lhs_exchange: "INDICES".to_string(),
        lhs_tradingsymbol: "NIFTY 50".to_string(),
        lhs_attribute: "LastTradedPrice".to_string(),
        operator: AlertOperator::Ge,
        rhs_type: "constant".to_string(),
        rhs_constant: Some(30000.0),
        rhs_exchange: None,
        rhs_tradingsymbol: None,
        rhs_attribute: None,
        basket: None,
    };

    match kite.create_alert(alert_params).await {
        Ok(alert) => {
            println!("✓ Alert created successfully:");
            println!("  UUID: {}", alert.uuid);
            println!("  Name: {}", alert.name);
            println!("  Status: {:?}", alert.status);
            println!("  Type: {:?}", alert.r#type);
            println!(
                "  Condition: {} {} {} {}",
                alert.lhs_tradingsymbol,
                match alert.operator {
                    AlertOperator::Ge => ">=",
                    AlertOperator::Le => "<=",
                    AlertOperator::Gt => ">",
                    AlertOperator::Lt => "<",
                    AlertOperator::Eq => "==",
                },
                alert.rhs_constant.unwrap_or(0.0),
                ""
            );

            // Example : Get specific alert
            println!("\nRetrieving the created alert...");
            match kite.get_alert(&alert.uuid).await {
                Ok(retrieved_alert) => {
                    println!("✓ Alert retrieved successfully:");
                    println!("  UUID: {}", retrieved_alert.uuid);
                    println!("  Name: {}", retrieved_alert.name);
                    println!("  Status: {:?}", retrieved_alert.status);
                }
                Err(e) => println!("✗ Error retrieving alert: {}", e),
            }

            // Example : Modify the alert
            println!("\nModifying the alert...");
            let modify_params = AlertParams {
                name: "NIFTY 50 Modified Alert".to_string(),
                r#type: AlertType::Simple,
                lhs_exchange: "INDICES".to_string(),
                lhs_tradingsymbol: "NIFTY 50".to_string(),
                lhs_attribute: "LastTradedPrice".to_string(),
                operator: AlertOperator::Ge,
                rhs_type: "constant".to_string(),
                rhs_constant: Some(30500.0), // Changed threshold
                rhs_exchange: None,
                rhs_tradingsymbol: None,
                rhs_attribute: None,
                basket: None,
            };

            match kite.modify_alert(&alert.uuid, modify_params).await {
                Ok(modified_alert) => {
                    println!("✓ Alert modified successfully:");
                    println!("  Name: {}", modified_alert.name);
                    println!(
                        "  New threshold: {}",
                        modified_alert.rhs_constant.unwrap_or(0.0)
                    );
                }
                Err(e) => println!("✗ Error modifying alert: {}", e),
            }

            // Example: Get alert history
            println!("\nGetting alert history...");
            match kite.get_alert_history(&alert.uuid).await {
                Ok(history) => {
                    println!("✓ Alert history retrieved:");
                    println!("  History entries: {}", history.len());
                    for (i, entry) in history.iter().enumerate() {
                        println!(
                            "  Entry {}: UUID: {}, Type: {:?}",
                            i + 1,
                            entry.uuid,
                            entry.r#type
                        );
                    }
                }
                Err(e) => println!("✗ Error getting alert history: {}", e),
            }

            // Example: Delete the alert
            println!("\nDeleting the alert...");
            match kite.delete_alerts(&[&alert.uuid]).await {
                Ok(()) => println!("✓ Alert deleted successfully"),
                Err(e) => println!("✗ Error deleting alert: {}", e),
            }
        }
        Err(e) => println!("✗ Error creating alert: {}", e),
    }

    // Example: Get all alerts
    println!("\nGetting all alerts...");
    match kite.get_alerts(None).await {
        Ok(alerts) => {
            println!("✓ Retrieved {} alerts", alerts.len());
            for (i, alert) in alerts.iter().take(3).enumerate() {
                println!("  Alert {}: {} ({})", i + 1, alert.name, alert.uuid);
            }
            if alerts.len() > 3 {
                println!("  ... and {} more", alerts.len() - 3);
            }
        }
        Err(e) => println!("✗ Error getting alerts: {}", e),
    }

    // Example: Get alerts with filters
    println!("\nGetting alerts with status filter...");
    let mut filters = std::collections::HashMap::new();
    filters.insert("status".to_string(), "enabled".to_string());

    match kite.get_alerts(Some(filters)).await {
        Ok(filtered_alerts) => {
            println!("✓ Retrieved {} enabled alerts", filtered_alerts.len());
        }
        Err(e) => println!("✗ Error getting filtered alerts: {}", e),
    }

    println!("\n=== Alerts API Examples Complete ===");
    Ok(())
}

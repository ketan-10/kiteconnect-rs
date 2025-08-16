use kiteconnect_rs::{
    KiteConnectBuilder,
    portfolio::{ConvertPositionParams, HoldingAuthParams, HoldingsAuthInstruments},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Portfolio API Examples ===\n");

    // Initialize KiteConnect (replace with your actual API key)
    let mut kite = KiteConnectBuilder::new("<api_key>").build()?;

    kite.set_access_token("<access_token>");

    // Example: Get holdings
    println!("==Fetching holdings...");
    match kite.get_holdings().await {
        Ok(holdings) => {
            println!("✓ Retrieved {} holdings", holdings.len());
            for (i, holding) in holdings.iter().take(3).enumerate() {
                println!(
                    "  Holding {}: {} ({}) - Qty: {}, PnL: {:.2}",
                    i + 1,
                    holding.tradingsymbol,
                    holding.exchange,
                    holding.quantity,
                    holding.pnl
                );
            }
            if holdings.len() > 3 {
                println!("  ... and {} more", holdings.len() - 3);
            }
        }
        Err(e) => println!("✗ Error fetching holdings: {}", e),
    }

    // Example: Get positions
    println!("\n==Fetching positions...");
    match kite.get_positions().await {
        Ok(positions) => {
            println!(
                "✓ Retrieved positions - Net: {}, Day: {}",
                positions.net.len(),
                positions.day.len()
            );

            if !positions.net.is_empty() {
                println!("  Net Positions:");
                for (i, position) in positions.net.iter().take(3).enumerate() {
                    println!(
                        "    {}: {} ({}) - Qty: {}, PnL: {:.2}",
                        i + 1,
                        position.tradingsymbol,
                        position.exchange,
                        position.quantity,
                        position.pnl
                    );
                }
            }

            if !positions.day.is_empty() {
                println!("  Day Positions:");
                for (i, position) in positions.day.iter().take(3).enumerate() {
                    println!(
                        "    {}: {} ({}) - Qty: {}, PnL: {:.2}",
                        i + 1,
                        position.tradingsymbol,
                        position.exchange,
                        position.quantity,
                        position.pnl
                    );
                }
            }
        }
        Err(e) => println!("✗ Error fetching positions: {}", e),
    }

    // Example: Convert position (example with realistic parameters)
    println!("\n==Converting position (if you have positions)...");

    // First check if we have any positions to convert
    if let Ok(positions) = kite.get_positions().await {
        if !positions.net.is_empty() {
            let first_position = &positions.net[0];

            // Example: Convert from MIS to CNC (intraday to delivery)
            let convert_params = ConvertPositionParams {
                exchange: first_position.exchange.clone(),
                tradingsymbol: first_position.tradingsymbol.clone(),
                old_product: first_position.product.clone(),
                new_product: if first_position.product == "MIS" {
                    "CNC".to_string()
                } else {
                    "MIS".to_string()
                },
                position_type: "day".to_string(),
                transaction_type: if first_position.quantity > 0 {
                    "BUY".to_string()
                } else {
                    "SELL".to_string()
                },
                quantity: first_position.quantity.abs(),
            };

            println!(
                "  Converting position: {} {} from {} to {}",
                convert_params.tradingsymbol,
                convert_params.quantity,
                convert_params.old_product,
                convert_params.new_product
            );

            match kite.convert_position(convert_params).await {
                Ok(success) => println!("✓ Position conversion successful: {}", success),
                Err(e) => println!("✗ Error converting position: {}", e),
            }
        } else {
            println!("  No positions available to convert");
        }
    } else {
        println!("  Could not fetch positions for conversion example");
    }

    // Example: Get auction instruments
    println!("\n==Fetching auction instruments...");
    match kite.get_auction_instruments().await {
        Ok(instruments) => {
            println!("✓ Retrieved {} auction instruments", instruments.len());
            for (i, instrument) in instruments.iter().take(3).enumerate() {
                println!(
                    "  Auction {}: {} ({}) - Qty: {}, Auction #: {}",
                    i + 1,
                    instrument.tradingsymbol,
                    instrument.exchange,
                    instrument.quantity,
                    instrument.auction_number
                );
            }
            if instruments.len() > 3 {
                println!("  ... and {} more", instruments.len() - 3);
            }
        }
        Err(e) => println!("✗ Error fetching auction instruments: {}", e),
    }

    // Example: Initiate holdings authorization
    println!("\n==Initiating holdings authorization...");

    // Example with specific instruments
    let auth_params_with_instruments = HoldingAuthParams {
        auth_type: "equity".to_string(),
        transfer_type: "pre".to_string(),
        exec_date: "2025-12-31".to_string(),
        instruments: Some(vec![
            HoldingsAuthInstruments {
                isin: "INE002A01018".to_string(), // Example ISIN for Reliance
                quantity: 10.0,
            },
            HoldingsAuthInstruments {
                isin: "INE009A01021".to_string(), // Example ISIN for Infosys
                quantity: 5.0,
            },
        ]),
    };

    match kite
        .initiate_holdings_auth(auth_params_with_instruments)
        .await
    {
        Ok(response) => {
            println!("✓ Holdings auth initiated successfully with specific instruments");
            println!("  Request ID: {}", response.request_id);
            if let Some(url) = response.redirect_url {
                println!("  Redirect URL: {}", url);
                println!(
                    "  Note: In a real app, redirect user to this URL to complete authorization"
                );
            }
        }
        Err(e) => println!("✗ Error initiating holdings auth with instruments: {}", e),
    }

    // Example without specific instruments (authorize all holdings)
    println!("\n==Initiating holdings authorization for all holdings...");
    let auth_params_all = HoldingAuthParams {
        auth_type: "equity".to_string(),
        transfer_type: "pre".to_string(),
        exec_date: "2025-12-31".to_string(),
        instruments: None, // Will authorize all holdings
    };

    match kite.initiate_holdings_auth(auth_params_all).await {
        Ok(response) => {
            println!("✓ Holdings auth initiated successfully for all holdings");
            println!("  Request ID: {}", response.request_id);
            if let Some(url) = response.redirect_url {
                println!("  Redirect URL: {}", url);
            }
        }
        Err(e) => println!("✗ Error initiating holdings auth for all: {}", e),
    }

    println!("\n=== Portfolio API Examples Complete ===");
    println!("Note: Some operations may fail with live data due to:");
    println!("  - No positions/holdings available");
    println!("  - Invalid conversion parameters");
    println!("  - Authorization requirements");
    println!("  - Market hours restrictions");

    Ok(())
}

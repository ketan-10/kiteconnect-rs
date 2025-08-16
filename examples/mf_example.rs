use kiteconnect_rs::KiteConnect;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize KiteConnect with your API key
    let mut kite = KiteConnect::builder("<api_key>").build()?;

    kite.set_access_token("<access_token>");

    // Example: Get all mutual fund orders
    println!("=== Getting MF Orders ===");
    match kite.get_mf_orders().await {
        Ok(orders) => {
            println!("{:#?}", orders);
        }
        Err(e) => println!("Error fetching MF orders: {}", e),
    }

    // Example: Get MF orders for a date range
    println!("\n=== Getting MF Orders by Date Range ===");
    match kite.get_mf_orders_by_date("2023-01-01", "2023-12-31").await {
        Ok(orders) => {
            println!("{:#?}", orders);
        }
        Err(e) => println!("Error fetching MF orders by date: {}", e),
    }

    // Example: Get mutual fund holdings
    println!("\n=== Getting MF Holdings ===");
    match kite.get_mf_holdings().await {
        Ok(holdings) => {
            println!("{:#?}", holdings);
        }
        Err(e) => println!("Error fetching MF holdings: {}", e),
    }

    // Example: Get all SIPs
    println!("\n=== Getting MF SIPs ===");
    match kite.get_mf_sips().await {
        Ok(sips) => {
            println!("{:#?}", sips);
        }
        Err(e) => println!("Error fetching SIPs: {}", e),
    }

    // Example 10: Get all allotted ISINs
    println!("\n=== Getting Allotted ISINs ===");
    match kite.get_mf_allotted_isins().await {
        Ok(isins) => {
            println!("allotted ISINs {:#?}", isins);
        }
        Err(e) => println!("Error fetching allotted ISINs: {}", e),
    }

    println!("\n=== MF Operations Demo Complete ===");
    Ok(())
}

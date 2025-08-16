use kiteconnect_rs::{KiteConnect, orders::OrderParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "<api_key>";
    let mut kite = KiteConnect::builder(&api_key).build()?;

    let access_token = "<access_token>";

    kite.set_access_token(&access_token);

    // Example: Get all orders
    println!("=== Getting All Orders ===");
    match kite.get_orders().await {
        Ok(orders) => {
            println!("{:#?}", orders);
        }
        Err(e) => println!("Failed to get orders: {:?}", e),
    }

    // Example: Get all trades
    println!("\n=== Getting All Trades ===");
    match kite.get_trades().await {
        Ok(trades) => {
            println!("{:#?}", trades);
        }
        Err(e) => println!("Failed to get trades: {:?}", e),
    }

    // Example: Place a new order
    println!("\n=== Placing a New Order ===");
    let order_params = OrderParams {
        exchange: Some("NSE".to_string()),
        tradingsymbol: Some("IDEA".to_string()),
        transaction_type: Some("BUY".to_string()),
        order_type: Some("LIMIT".to_string()),
        quantity: Some(1),
        price: Some(6.52),
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
        tag: Some("example-order".to_string()),
        validity_ttl: None,
    };

    match kite.place_order("regular", order_params).await {
        Ok(response) => {
            println!("Order placed successfully! Order ID: {}", response.order_id);

            // Example: Get order history for the placed order
            println!("\n=== Getting Order History ===");
            match kite.get_order_history(&response.order_id).await {
                Ok(history) => {
                    println!("Order history has {} entries", history.len());
                    for order in &history {
                        println!(
                            "Status: {}, Timestamp: {:?}",
                            order.status, order.order_timestamp
                        );
                    }
                }
                Err(e) => println!("Failed to get order history: {:?}", e),
            }

            // Example: Modify the order
            println!("\n=== Modifying Order ===");
            let modify_params = OrderParams {
                price: Some(6.54), // Increase price
                quantity: Some(2), // Double the quantity
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
                tag: Some("modified-order".to_string()),
                validity_ttl: None,
            };

            match kite
                .modify_order("regular", &response.order_id, modify_params)
                .await
            {
                Ok(modify_response) => {
                    println!(
                        "Order modified successfully! Order ID: {}",
                        modify_response.order_id
                    );
                }
                Err(e) => println!("Failed to modify order: {:?}", e),
            }

            // Example: Get trades for the specific order
            println!("\n=== Getting Order Trades ===");
            match kite.get_order_trades(&response.order_id).await {
                Ok(order_trades) => {
                    if order_trades.is_empty() {
                        println!(
                            "No trades found for this order (order might not be executed yet)"
                        );
                    } else {
                        println!("Found {} trades for this order", order_trades.len());
                        for trade in &order_trades {
                            println!(
                                "Trade ID: {}, Quantity: {}, Price: {}",
                                trade.trade_id, trade.quantity, trade.average_price
                            );
                        }
                    }
                }
                Err(e) => println!("Failed to get order trades: {:?}", e),
            }

            // Example: Cancel the order
            println!("\n=== Cancelling Order ===");
            match kite.cancel_order("regular", &response.order_id, None).await {
                Ok(cancel_response) => {
                    println!(
                        "Order cancelled successfully! Order ID: {}",
                        cancel_response.order_id
                    );
                }
                Err(e) => println!("Failed to cancel order: {:?}", e),
            }
        }
        Err(e) => println!("Failed to place order: {:?}", e),
    }

    // Example: Demonstrate exit_order (alias for cancel_order)
    println!("\n=== Using Exit Order (for bracket orders, etc.) ===");
    let hypothetical_order_id = "123456789";
    match kite.exit_order("co", hypothetical_order_id, None).await {
        Ok(exit_response) => {
            println!("{:#?}", exit_response);
        }
        Err(e) => println!("Expected failure to exit hypothetical order: {:?}", e),
    }

    // Example: Different order varieties
    println!("\n=== Different Order Varieties ===");

    // Market order
    let market_order_params = OrderParams {
        exchange: Some("NSE".to_string()),
        tradingsymbol: Some("IDEA".to_string()),
        transaction_type: Some("BUY".to_string()),
        order_type: Some("MARKET".to_string()),
        quantity: Some(1),
        price: None, // No price for market orders
        product: Some("MIS".to_string()),
        validity: Some("DAY".to_string()),
        disclosed_quantity: None,
        trigger_price: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: Some("market-order-example".to_string()),
        validity_ttl: None,
    };

    match kite.place_order("regular", market_order_params).await {
        Ok(response) => println!("Market order placed! Order ID: {:#?}", response),
        Err(e) => println!("Expected failure for demo market order: {:?}", e),
    }

    // Stop-loss order
    let sl_order_params = OrderParams {
        exchange: Some("NSE".to_string()),
        tradingsymbol: Some("IDEA".to_string()),
        transaction_type: Some("SELL".to_string()),
        order_type: Some("SL".to_string()),
        quantity: Some(1),
        price: Some(6.28),        // SL price
        trigger_price: Some(6.3), // Trigger should be above SL price for sell
        product: Some("MIS".to_string()),
        validity: Some("DAY".to_string()),
        disclosed_quantity: None,
        squareoff: None,
        stoploss: None,
        trailing_stoploss: None,
        iceberg_legs: None,
        iceberg_quantity: None,
        auction_number: None,
        tag: Some("stop-loss-example".to_string()),
        validity_ttl: None,
    };

    match kite.place_order("regular", sl_order_params).await {
        Ok(response) => println!("Stop-loss order placed! Order ID: {:#?}", response),
        Err(e) => println!("Expected failure for demo stop-loss order: {:?}", e),
    }

    println!("\n=== Order Operations Demo Complete ===");
    Ok(())
}

use std::time::Duration;

use kiteconnect_rs::ticker::{Mode, Ticker, TickerEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ticker and get handle
    let (ticker, handle) = Ticker::builder("<api_key>", "<access_token>")
        .auto_reconnect(true)
        .reconnect_max_retries(10)
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    // Subscribe to events before starting
    let event_receiver = handle.subscribe_events();

    // Clone handle for event handler task
    let event_handle_clone = handle.clone();

    // Start the WebSocket server in a background task
    let serve_task = tokio::spawn(async move {
        if let Err(e) = ticker.serve().await {
            eprintln!("Ticker serve error: {}", e);
        }
    });

    // Handle events and manage subscriptions
    let event_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                TickerEvent::Connect => {
                    println!("Connected! Subscribing to instruments...");

                    // Now we can subscribe using the handle without blocking
                    let tokens = vec![256265, 738561]; // NIFTY 50 and RELIANCE

                    if let Err(e) = event_handle_clone.subscribe(tokens.clone()).await {
                        eprintln!("Subscribe error: {}", e);
                    } else {
                        println!("Subscribed to tokens: {:?}", tokens);
                    }

                    // Set mode to Full for detailed data
                    if let Err(e) = event_handle_clone
                        .set_mode(Mode::Full, tokens.clone())
                        .await
                    {
                        eprintln!("Set mode error: {}", e);
                    }

                    // Later, we can add more subscriptions dynamically
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    let more_tokens = vec![341249]; // HDFC Bank
                    if let Err(e) = event_handle_clone.subscribe(more_tokens.clone()).await {
                        eprintln!("Subscribe error: {}", e);
                    } else {
                        println!("Added more subscriptions: {:?}", more_tokens);
                    }
                }
                TickerEvent::Tick(tick) => {
                    println!(
                        "Tick: {} - Price: {:.2}, Volume: {}",
                        tick.instrument_token, tick.last_price, tick.volume_traded
                    );
                    // println!(" Tick: {:#?}", tick);
                }
                TickerEvent::Error(e) => {
                    eprintln!("Error: {}", e);
                }
                TickerEvent::Close(code, reason) => {
                    println!("Connection closed: {} - {}", code, reason);
                    break;
                }
                TickerEvent::Reconnect(attempt, delay) => {
                    println!("Reconnecting (attempt {}), waiting {:?}...", attempt, delay);
                }
                _ => {}
            }
        }
    });

    // You can also use the handle from the main thread
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Example: Unsubscribe from a token
    println!("Unsubscribing from token 341249...");
    if let Err(e) = handle.unsubscribe(vec![341249]).await {
        eprintln!("Unsubscribe error: {}", e);
    }

    // Example: Change mode for remaining tokens
    println!("Changing mode to Quote for remaining tokens...");
    if let Err(e) = handle.set_mode(Mode::Quote, vec![256265, 738561]).await {
        eprintln!("Set mode error: {}", e);
    }

    // Wait for tasks to complete
    tokio::select! {
        _ = serve_task => println!("Serve task completed"),
        _ = event_task => println!("Event handler completed"),
    }

    Ok(())
}

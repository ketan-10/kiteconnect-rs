# The Kite Connect API Rust client

A Rust client for communicating with the Kite Connect API.

Kite Connect is a set of REST-like APIs that expose many capabilities required
to build a complete investment and trading platform. Execute orders in real
time, manage user portfolio, stream live market data (WebSockets), and more,
with the simple HTTP API collection.

Licensed under the MIT License.

## Documentation

- [Kite Connect HTTP API documentation](https://kite.trade/docs/connect/v3)
- [Rust docs.rs documentation](https://docs.rs/kiteconnect-rs)
- [kiteconnect-rs crate](https://crates.io/crates/kiteconnect-rs)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kiteconnect-rs = "0.1.0"
```

## Features

- **Async/Await Support**: Built with modern async Rust using tokio
- **Type Safety**: Full type definitions for all API responses
- **WebSocket Ticker**: Real-time market data streaming
- **Comprehensive API Coverage**: All Kite Connect endpoints supported
- **Error Handling**: Robust error handling with custom error types

## API Usage

```rust
use kiteconnect_rs::{KiteConnect, orders::OrderParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "your_api_key";
    let mut kite = KiteConnect::builder(api_key).build()?;

    // Login URL from which request token can be obtained
    println!("{}", kite.get_login_url());

    // After obtaining request token from the login flow
    let request_token = "your_request_token";
    let api_secret = "your_api_secret";

    // Generate user session
    let session = kite.generate_session(&request_token, &api_secret).await?;

    // Get user margins
    let margins = kite.get_user_margins().await?;
    println!("Margins: {:#?}", margins);

    // Place an order
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

    let order_response = kite.place_order(order_params).await?;
    println!("Order placed: {:#?}", order_response);

    Ok(())
}
```

## Kite Ticker Usage

```rust
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
    let mut event_receiver = handle.subscribe_events();

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
```

## Examples

Check the [examples folder](examples/) for comprehensive examples covering:

- **Basic API operations**: Authentication, user profile, margins
- **Order management**: Place, modify, cancel orders
- **Portfolio operations**: Holdings, positions, P&L
- **Market data**: Quotes, historical data, instruments
- **Mutual funds**: Orders, holdings, SIPs
- **Alerts & GTT**: Create and manage alerts
- **WebSocket ticker**: Real-time market data streaming

You can run examples with:

```bash
cargo run --example order_operations
cargo run --example ticker_example
cargo run --example portfolio_example
```

## Development

### Setup

```bash
git clone https://github.com/ketan-10/kiteconnect-rs.git
cd kiteconnect-rs
```

### Run tests

```bash
cargo test
```

### Run integration tests

```bash
cargo test --test integration_tests
```

### Generate documentation

```bash
cargo doc --open
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


- Frequently used commands 
    - `cargo run --example margins_example`
    - `cargo doc --open --no-deps`
    - `cargo test test_user_profile_parsing --test unit_tests -- --nocapture`
    - `cargo test test_generate_session test_renew_access_token --test integration_tests`
    - `RUST_BACKTRACE=1 cargo run --example margins_example`
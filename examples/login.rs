//! Login Helper
//!
//! Usage: cargo run --example login

use kiteconnect_rs::KiteConnect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // dotenvy::from_filename("examples/.env").ok();

    dotenvy::dotenv().ok();

    let api_key = std::env::var("KITE_API_KEY").expect("KITE_API_KEY not set");
    let api_secret = std::env::var("KITE_API_SECRET").expect("KITE_API_SECRET not set");

    let mut kite = KiteConnect::builder(&api_key).build()?;

    println!("Login URL: {}", kite.get_login_url());
    println!("\nEnter request_token: ");

    let mut request_token = String::new();
    std::io::stdin().read_line(&mut request_token)?;

    let access_token = kite
        .generate_session(request_token.trim(), &api_secret)
        .await?
        .access_token;

    println!("Access Token: {}", access_token);

    Ok(())
}

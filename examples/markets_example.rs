use kiteconnect_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "<api_key>";
    let mut kite = KiteConnect::builder(&api_key).build()?;

    // hardcoded access token for testing (expires in 24 hours)
    let access_token = "<access_token>";

    // Set access token (you would get this from the authentication flow)
    kite.set_access_token(&access_token);

    // Example: Get full quote for instruments
    let instruments = vec!["NSE:INFY", "NSE:TCS", "NSE:RELIANCE"];

    match kite.get_quote(&instruments).await {
        Ok(quotes) => {
            println!("quotes:: \n {:#?}", quotes);
        }
        Err(e) => {
            eprintln!("Error fetching quotes: {}", e);
            eprintln!("\nStack trace:");
            e.print_backtrace();
        }
    }

    // Example: Get LTP (Last Traded Price) only
    match kite.get_ltp(&instruments).await {
        Ok(ltps) => {
            println!("ltps:: \n {:#?}", ltps);
        }
        Err(e) => {
            eprintln!("Error fetching LTP: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get OHLC data
    match kite.get_ohlc(&instruments).await {
        Ok(ohlc_data) => {
            println!("ohlc_data:: \n {:#?}", ohlc_data);
        }
        Err(e) => {
            eprintln!("Error fetching OHLC: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get historical data
    let instrument_token = 408065; // INFY token (example)
    let interval = "minute"; // Can be: minute, day, 3minute, 5minute, 10minute, 15minute, 30minute, 60minute
    let from_date = "2024-01-01 09:15:00";
    let to_date = "2024-01-01 15:30:00";

    match kite
        .get_historical_data(
            instrument_token,
            interval,
            from_date,
            to_date,
            false, // continuous
            false, // include OI
        )
        .await
    {
        Ok(historical_data) => {
            println!("historical_data:: \n {:#?}", historical_data);
        }
        Err(e) => {
            eprintln!("Error fetching historical data: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get all instruments
    match kite.get_instruments().await {
        Ok(instruments) => {
            println!("instruments:: \n {:#?}", instruments);
        }
        Err(e) => {
            eprintln!("Error fetching instruments: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get instruments by exchange
    match kite.get_instruments_by_exchange("NSE").await {
        Ok(instruments) => {
            println!("instruments:: \n {:#?}", instruments);
        }
        Err(e) => {
            eprintln!("Error fetching NSE instruments: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get mutual fund instruments
    match kite.get_mf_instruments().await {
        Ok(mf_instruments) => {
            println!("mf_instruments:: \n {:#?}", mf_instruments);
        }
        Err(e) => {
            eprintln!("Error fetching MF instruments: {}", e);
            e.print_backtrace();
        }
    }

    Ok(())
}

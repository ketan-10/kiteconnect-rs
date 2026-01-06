use kiteconnect_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("KITE_API_KEY").expect("KITE_API_KEY not set");
    let access_token = std::env::var("KITE_ACCESS_TOKEN").expect("KITE_ACCESS_TOKEN not set");

    let mut kite = KiteConnect::builder(&api_key).build()?;
    kite.set_access_token(&access_token);

    // Example: Get order margins for a single order
    let order_param = OrderMarginParam {
        exchange: "NSE".to_string(),
        trading_symbol: "INFY".to_string(),
        transaction_type: "BUY".to_string(),
        variety: "regular".to_string(),
        product: "CNC".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        price: None,
        trigger_price: None,
    };

    // Get compact order margins
    match kite
        .get_order_margins(GetMarginParams {
            order_params: vec![order_param.clone()],
            compact: true,
        })
        .await
    {
        Ok(margins) => {
            println!("{:#?}", margins);
        }
        Err(e) => {
            eprintln!("Error fetching order margins: {}", e);
            eprintln!("\nStack trace:");
            e.print_backtrace();
        }
    }

    // Get detailed order margins
    match kite
        .get_order_margins(GetMarginParams {
            order_params: vec![order_param.clone()],
            compact: false,
        })
        .await
    {
        Ok(margins) => {
            println!("{:#?}", margins);
        }
        Err(e) => {
            eprintln!("Error fetching detailed order margins: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get basket margins for multiple orders
    let order_params = vec![
        OrderMarginParam {
            exchange: "NSE".to_string(),
            trading_symbol: "INFY".to_string(),
            transaction_type: "BUY".to_string(),
            variety: "regular".to_string(),
            product: "CNC".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            price: None,
            trigger_price: None,
        },
        OrderMarginParam {
            exchange: "NSE".to_string(),
            trading_symbol: "TCS".to_string(),
            transaction_type: "BUY".to_string(),
            variety: "regular".to_string(),
            product: "CNC".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            price: Some(3000.0),
            trigger_price: None,
        },
    ];

    match kite
        .get_basket_margins(GetBasketParams {
            order_params: order_params.clone(),
            compact: false,
            consider_positions: true,
        })
        .await
    {
        Ok(basket) => {
            println!("{:#?}", basket);
        }
        Err(e) => {
            eprintln!("Error getting basket margins: {}", e);
            e.print_backtrace();
        }
    }

    // Example: Get order charges for executed orders
    let charges_params = vec![OrderChargesParam {
        order_id: "order123".to_string(),
        exchange: "NSE".to_string(),
        trading_symbol: "INFY".to_string(),
        transaction_type: "BUY".to_string(),
        variety: "regular".to_string(),
        product: "CNC".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        average_price: 1500.0,
    }];

    match kite
        .get_order_charges(GetChargesParams {
            order_params: charges_params,
        })
        .await
    {
        Ok(charges) => {
            println!("{:#?}", charges);
        }
        Err(e) => {
            eprintln!("Error getting order charges: {}", e);
            e.print_backtrace();
        }
    }

    Ok(())
}

//! WASM Ticker Example
//!
//! This example demonstrates how to use kiteconnect-rs in a browser environment.
//!
//! Build with: wasm-pack build --target web
//! Then open index.html in a browser.

use kiteconnect_rs::ticker::{Mode, Ticker, TickerEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::console;
use web_time::Duration;

/// Initialize panic hook for better error messages in browser console
/// Also exposes functions to window.wasm for the HTML to use
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    log("WASM module initialized");

    // Expose functions to window.wasm for HTML to access
    if let Some(window) = web_sys::window() {
        let wasm_obj = js_sys::Object::new();

        // Mark as ready
        js_sys::Reflect::set(&wasm_obj, &JsValue::from_str("ready"), &JsValue::from(true)).ok();

        // Set up function wrappers using closures
        let start_ticker_fn = Closure::wrap(Box::new(|api_key: String, access_token: String, tokens: String| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = start_ticker(api_key, access_token, tokens).await {
                    log_error(&format!("start_ticker error: {:?}", e));
                }
            });
        }) as Box<dyn Fn(String, String, String)>);

        js_sys::Reflect::set(&wasm_obj, &JsValue::from_str("start_ticker"), start_ticker_fn.as_ref()).ok();
        start_ticker_fn.forget(); // Don't drop the closure

        let clear_output_fn = Closure::wrap(Box::new(|| {
            clear_output();
        }) as Box<dyn Fn()>);

        js_sys::Reflect::set(&wasm_obj, &JsValue::from_str("clear_output"), clear_output_fn.as_ref()).ok();
        clear_output_fn.forget();

        let get_login_url_fn = Closure::wrap(Box::new(|api_key: String| -> String {
            get_login_url(&api_key)
        }) as Box<dyn Fn(String) -> String>);

        js_sys::Reflect::set(&wasm_obj, &JsValue::from_str("get_login_url"), get_login_url_fn.as_ref()).ok();
        get_login_url_fn.forget();

        // Attach to window
        js_sys::Reflect::set(&window, &JsValue::from_str("wasm"), &wasm_obj).ok();
        log("window.wasm initialized");
    }
}

/// Log a message to the browser console
fn log(msg: &str) {
    console::log_1(&JsValue::from_str(msg));
}

/// Log an error to the browser console
fn log_error(msg: &str) {
    console::error_1(&JsValue::from_str(msg));
}

/// Append a message to the output div in the HTML page
fn append_to_output(msg: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(output) = document.get_element_by_id("output") {
                let current = output.inner_html();
                let timestamp = js_sys::Date::new_0().to_locale_time_string("en-US");
                let new_content = format!(
                    "{}<div class=\"log-entry\"><span class=\"timestamp\">[{}]</span> {}</div>",
                    current,
                    timestamp.as_string().unwrap_or_default(),
                    msg
                );
                output.set_inner_html(&new_content);
                // Auto-scroll to bottom
                if let Some(element) = output.dyn_ref::<web_sys::HtmlElement>() {
                    element.set_scroll_top(element.scroll_height());
                }
            }
        }
    }
}

/// Clear the output div
#[wasm_bindgen]
pub fn clear_output() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(output) = document.get_element_by_id("output") {
                output.set_inner_html("");
            }
        }
    }
}

/// Update connection status in the UI
fn set_status(status: &str, class: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(status_el) = document.get_element_by_id("status") {
                status_el.set_inner_html(status);
                status_el.set_class_name(&format!("status {}", class));
            }
        }
    }
}

/// Start the ticker with the provided credentials
/// This is called from JavaScript when the user clicks "Connect"
#[wasm_bindgen]
pub async fn start_ticker(api_key: String, access_token: String, tokens_str: String) -> Result<(), JsValue> {
    log(&format!("Starting ticker with API key: {}...", &api_key[..api_key.len().min(8)]));
    append_to_output(&format!("Starting ticker connection..."));
    set_status("Connecting...", "connecting");

    // Parse tokens from comma-separated string
    let tokens: Vec<u32> = tokens_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if tokens.is_empty() {
        let err = "No valid instrument tokens provided";
        log_error(err);
        append_to_output(&format!("<span class=\"error\">{}</span>", err));
        set_status("Error", "error");
        return Err(JsValue::from_str(err));
    }

    append_to_output(&format!("Tokens to subscribe: {:?}", tokens));

    // Create ticker
    let (ticker, handle) = Ticker::builder(&api_key, &access_token)
        .auto_reconnect(true)
        .reconnect_max_retries(5)
        .reconnect_max_delay(Duration::from_secs(30))
        .build()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Subscribe to events
    let event_receiver = handle.subscribe_events();
    let handle_clone = handle.clone();
    let tokens_clone = tokens.clone();

    // Spawn event handler
    wasm_bindgen_futures::spawn_local(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                TickerEvent::Connect => {
                    log("Connected to Kite WebSocket");
                    append_to_output("<span class=\"success\">Connected to Kite WebSocket!</span>");
                    set_status("Connected", "connected");

                    // Subscribe to tokens
                    if let Err(e) = handle_clone.subscribe(tokens_clone.clone()).await {
                        log_error(&format!("Subscribe error: {}", e));
                        append_to_output(&format!("<span class=\"error\">Subscribe error: {}</span>", e));
                    } else {
                        append_to_output(&format!("Subscribed to {} instruments", tokens_clone.len()));
                    }

                    // Set mode to Full for detailed data
                    if let Err(e) = handle_clone.set_mode(Mode::Full, tokens_clone.clone()).await {
                        log_error(&format!("Set mode error: {}", e));
                    } else {
                        append_to_output("Mode set to Full");
                    }
                }
                TickerEvent::Tick(tick) => {
                    let msg = format!(
                        "<span class=\"tick\">Tick:</span> Token: <b>{}</b> | Price: <b>{:.2}</b> | Volume: {} | Change: {:.2}",
                        tick.instrument_token,
                        tick.last_price,
                        tick.volume_traded,
                        tick.net_change
                    );
                    append_to_output(&msg);
                    log(&format!(
                        "Tick: {} - Price: {:.2}",
                        tick.instrument_token, tick.last_price
                    ));
                }
                TickerEvent::Error(e) => {
                    log_error(&format!("Ticker error: {}", e));
                    append_to_output(&format!("<span class=\"error\">Error: {}</span>", e));
                    set_status("Error", "error");
                }
                TickerEvent::Close(code, reason) => {
                    let msg = format!("Connection closed: {} - {}", code, reason);
                    log(&msg);
                    append_to_output(&format!("<span class=\"warning\">{}</span>", msg));
                    set_status("Disconnected", "disconnected");
                }
                TickerEvent::Reconnect(attempt, delay) => {
                    let msg = format!("Reconnecting (attempt {}), waiting {:?}...", attempt, delay);
                    log(&msg);
                    append_to_output(&format!("<span class=\"warning\">{}</span>", msg));
                    set_status(&format!("Reconnecting ({})", attempt), "connecting");
                }
                TickerEvent::NoReconnect(attempts) => {
                    let msg = format!("Max reconnection attempts ({}) reached", attempts);
                    log_error(&msg);
                    append_to_output(&format!("<span class=\"error\">{}</span>", msg));
                    set_status("Failed", "error");
                }
                TickerEvent::OrderUpdate(order) => {
                    let msg = format!("Order update: {} - {}", order.order_id, order.status);
                    log(&msg);
                    append_to_output(&format!("<span class=\"order\">{}</span>", msg));
                }
                TickerEvent::Message(_) => {
                    // Raw message, usually not needed for display
                }
            }
        }
    });

    // Spawn ticker serve
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = ticker.serve().await {
            log_error(&format!("Ticker serve error: {}", e));
            append_to_output(&format!("<span class=\"error\">Ticker error: {}</span>", e));
            set_status("Error", "error");
        }
    });

    Ok(())
}

/// Get the login URL for Kite Connect
#[wasm_bindgen]
pub fn get_login_url(api_key: &str) -> String {
    format!("https://kite.zerodha.com/connect/login?v=3&api_key={}", api_key)
}

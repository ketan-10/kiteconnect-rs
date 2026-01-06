# WASM Ticker Example

This example demonstrates how to use `kiteconnect-rs` in a browser environment using WebAssembly.

## Prerequisites

1. **trunk** (recommended): `cargo install trunk`
   Or **wasm-pack**: `cargo install wasm-pack`
2. **Kite Connect API credentials**: Get them from [Kite Connect](https://kite.trade/)

## Running with Trunk (Recommended)

Trunk handles building and serving with hot-reload:

```bash
cd examples/wasm-ticker
trunk serve
```

This will:
- Build the WASM module
- Start a dev server on http://localhost:8080
- Auto-open your browser
- Hot-reload on changes

## Alternative: wasm-pack

If you prefer wasm-pack:

```bash
cd examples/wasm-ticker
wasm-pack build --target web
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

## Usage

1. Enter your **API Key** from Kite Connect
2. Click the login link to authenticate and get an **Access Token**
3. Enter instrument tokens (comma-separated)
   - Example tokens: `256265` (NIFTY 50), `738561` (RELIANCE), `341249` (HDFC Bank)
4. Click **Connect** to start receiving live market data

## Common Instrument Tokens

| Token | Instrument |
|-------|-----------|
| 256265 | NIFTY 50 |
| 260105 | NIFTY BANK |
| 738561 | RELIANCE |
| 341249 | HDFC BANK |
| 2953217 | TCS |
| 408065 | INFY |

## Troubleshooting

### "Failed to load WASM module"
Make sure you ran `wasm-pack build --target web` and are serving via HTTP (not file://).

### "Connection error" or "WebSocket error"
- Check that your access token is valid (they expire daily)
- Ensure you're using the correct API key
- Check browser console for detailed error messages

### CORS errors
The Kite WebSocket endpoint should work from any origin. If you see CORS errors, check your network/firewall settings.

## Architecture

```
Browser
  |
  +-- index.html (UI)
  |
  +-- pkg/wasm_ticker_example.js (JS bindings)
  |
  +-- pkg/wasm_ticker_example_bg.wasm (Rust compiled to WASM)
        |
        +-- kiteconnect-rs (Ticker, WebSocket handling)
              |
              +-- gloo-net (WASM WebSocket)
              +-- web-time (Browser-compatible time)
```

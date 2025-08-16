use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::{
    KiteConnect,
    constants::Endpoints,
    models::{Depth, KiteConnectError, OHLC, time},
};

/// Custom deserializer to convert integer (0/1) to boolean
fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"0 or 1",
        )),
    }
}

/// Quote represents the full quote response for a single instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub instrument_token: u32,
    #[serde(default)]
    pub timestamp: time::Time,
    pub last_price: f64,
    pub last_quantity: u32,
    #[serde(default)]
    pub last_trade_time: time::Time,
    pub average_price: f64,
    pub volume: u32,
    pub buy_quantity: u32,
    pub sell_quantity: u32,
    pub ohlc: OHLC,
    pub net_change: f64,
    pub oi: f64,
    pub oi_day_high: f64,
    pub oi_day_low: f64,
    pub lower_circuit_limit: f64,
    pub upper_circuit_limit: f64,
    pub depth: Depth,
}

/// Quote represents a map of instrument symbols to their quote data.
pub type Quote = HashMap<String, QuoteData>;

/// QuoteOHLCData represents OHLC quote response for a single instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteOHLCData {
    pub instrument_token: u32,
    pub last_price: f64,
    pub ohlc: OHLC,
}

/// QuoteOHLC represents a map of instrument symbols to their OHLC data.
pub type QuoteOHLC = HashMap<String, QuoteOHLCData>;

/// QuoteLTPData represents last price quote response for a single instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteLTPData {
    pub instrument_token: u32,
    pub last_price: f64,
}

/// QuoteLTP represents a map of instrument symbols to their LTP data.
pub type QuoteLTP = HashMap<String, QuoteLTPData>;

/// HistoricalData represents individual historical data response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    #[serde(default)]
    pub date: time::Time,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u32,
    pub oi: u32,
}

/// HistoricalDataResponse represents the response wrapper for historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoricalDataResponse {
    pub candles: Vec<Vec<serde_json::Value>>,
}

/// HistoricalDataParams represents parameters for historical data requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataParams {
    pub from: String,
    pub to: String,
    pub continuous: bool,
    pub oi: bool,
}

/// Instrument represents individual instrument response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub instrument_token: u32,
    pub exchange_token: u32,
    pub tradingsymbol: String,
    pub name: String,
    pub last_price: f64,
    #[serde(default)]
    pub expiry: time::Time,
    pub strike: f64,
    pub tick_size: f64,
    pub lot_size: f64,
    pub instrument_type: String,
    pub segment: String,
    pub exchange: String,
}

/// Instruments represents list of instruments.
pub type Instruments = Vec<Instrument>;

/// MFInstrument represents individual mutual fund instrument response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFInstrument {
    pub tradingsymbol: String,
    pub name: String,
    pub last_price: f64,
    pub amc: String,
    #[serde(deserialize_with = "bool_from_int")]
    pub purchase_allowed: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub redemption_allowed: bool,
    pub minimum_purchase_amount: f64,
    pub purchase_amount_multiplier: f64,
    pub minimum_additional_purchase_amount: f64,
    pub minimum_redemption_quantity: f64,
    pub redemption_quantity_multiplier: f64,
    pub dividend_type: String,
    pub scheme_type: String,
    pub plan: String,
    pub settlement_type: String,
    #[serde(default)]
    pub last_price_date: time::Time,
}

/// MFInstruments represents list of mutual fund instruments.
pub type MFInstruments = Vec<MFInstrument>;

impl KiteConnect {
    /// Gets quote for given instruments.
    pub async fn get_quote(&self, instruments: &[&str]) -> Result<Quote, KiteConnectError> {
        let params = instruments
            .iter()
            .map(|&inst| ("i".to_string(), inst.to_string()))
            .collect();

        self.get_with_query(Endpoints::GET_QUOTE, params).await
    }

    /// Gets LTP for given instruments.
    pub async fn get_ltp(&self, instruments: &[&str]) -> Result<QuoteLTP, KiteConnectError> {
        let params = instruments
            .iter()
            .map(|&inst| ("i".to_string(), inst.to_string()))
            .collect();

        self.get_with_query(Endpoints::GET_LTP, params).await
    }

    /// Gets OHLC for given instruments.
    pub async fn get_ohlc(&self, instruments: &[&str]) -> Result<QuoteOHLC, KiteConnectError> {
        let params = instruments
            .iter()
            .map(|&inst| ("i".to_string(), inst.to_string()))
            .collect();

        self.get_with_query(Endpoints::GET_OHLC, params).await
    }

    /// Gets historical data for a given instrument.
    pub async fn get_historical_data(
        &self,
        instrument_token: u32,
        interval: &str,
        from_date: &str,
        to_date: &str,
        continuous: bool,
        oi: bool,
    ) -> Result<Vec<HistoricalData>, KiteConnectError> {
        let endpoint = &Endpoints::GET_HISTORICAL
            .replace("{instrument_token}", &instrument_token.to_string())
            .replace("{interval}", interval);

        let mut params = HashMap::new();
        params.insert("from".to_string(), from_date.to_string());
        params.insert("to".to_string(), to_date.to_string());
        params.insert(
            "continuous".to_string(),
            if continuous { "1" } else { "0" }.to_string(),
        );
        params.insert("oi".to_string(), if oi { "1" } else { "0" }.to_string());

        let response: HistoricalDataResponse = self.get_with_query(endpoint, params).await?;
        self.format_historical_data(response)
    }

    /// Formats historical data response into structured data.
    fn format_historical_data(
        &self,
        response: HistoricalDataResponse,
    ) -> Result<Vec<HistoricalData>, KiteConnectError> {
        let mut data = Vec::new();

        for candle in response.candles {
            if candle.len() < 6 {
                return Err(KiteConnectError::other(
                    "Invalid candle data format".to_string(),
                ));
            }

            let date_str = candle[0]
                .as_str()
                .ok_or_else(|| KiteConnectError::other("Invalid date format".to_string()))?;

            let open = candle[1]
                .as_f64()
                .ok_or_else(|| KiteConnectError::other("Invalid open price".to_string()))?;

            let high = candle[2]
                .as_f64()
                .ok_or_else(|| KiteConnectError::other("Invalid high price".to_string()))?;

            let low = candle[3]
                .as_f64()
                .ok_or_else(|| KiteConnectError::other("Invalid low price".to_string()))?;

            let close = candle[4]
                .as_f64()
                .ok_or_else(|| KiteConnectError::other("Invalid close price".to_string()))?;

            let volume = candle[5]
                .as_f64()
                .ok_or_else(|| KiteConnectError::other("Invalid volume".to_string()))?
                as u32;

            // OI is optional (7th element)
            let oi = if candle.len() > 6 {
                candle[6].as_f64().unwrap_or(0.0) as u32
            } else {
                0
            };

            // Parse date - handle different timezone formats
            let parsed_date = if date_str.len() > 19 {
                // Try with colon in timezone first (RFC3339 standard)
                let date_with_colon = if date_str.ends_with("+0530") {
                    date_str.replace("+0530", "+05:30")
                } else if date_str.ends_with("-0530") {
                    date_str.replace("-0530", "-05:30")
                } else {
                    date_str.to_string()
                };

                chrono::DateTime::parse_from_rfc3339(&date_with_colon)
                    .or_else(|_| chrono::DateTime::parse_from_rfc3339(date_str))
            } else {
                chrono::DateTime::parse_from_rfc3339(date_str)
            };

            let date = parsed_date
                .map_err(|e| {
                    KiteConnectError::other(format!("Failed to parse date '{}': {}", date_str, e))
                })?
                .with_timezone(&chrono::Utc);

            data.push(HistoricalData {
                date: time::Time::new(date),
                open,
                high,
                low,
                close,
                volume,
                oi,
            });
        }

        Ok(data)
    }

    /// Gets all instruments.
    pub async fn get_instruments(&self) -> Result<Instruments, KiteConnectError> {
        let csv_text: String = self.get(Endpoints::GET_INSTRUMENTS).await?;
        let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
        let mut instruments = Vec::new();

        for result in reader.deserialize() {
            let instrument: Instrument =
                result.map_err(|e| KiteConnectError::other(format!("CSV parsing error: {}", e)))?;
            instruments.push(instrument);
        }

        Ok(instruments)
    }

    /// Gets instruments by exchange.
    pub async fn get_instruments_by_exchange(
        &self,
        exchange: &str,
    ) -> Result<Instruments, KiteConnectError> {
        let endpoint = &Endpoints::GET_INSTRUMENTS_EXCHANGE.replace("{exchange}", exchange);
        let csv_text: String = self.get(endpoint).await?;
        let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
        let mut instruments = Vec::new();

        for result in reader.deserialize() {
            let instrument: Instrument =
                result.map_err(|e| KiteConnectError::other(format!("CSV parsing error: {}", e)))?;
            instruments.push(instrument);
        }

        Ok(instruments)
    }

    /// Gets all mutual fund instruments.
    pub async fn get_mf_instruments(&self) -> Result<MFInstruments, KiteConnectError> {
        let csv_text: String = self.get(Endpoints::GET_MF_INSTRUMENTS).await?;
        let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
        let mut instruments = Vec::new();

        for result in reader.deserialize() {
            let instrument: MFInstrument =
                result.map_err(|e| KiteConnectError::other(format!("CSV parsing error: {}", e)))?;
            instruments.push(instrument);
        }

        Ok(instruments)
    }
}

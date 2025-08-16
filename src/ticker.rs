use crate::models::time::Time;
use crate::models::{DepthItem, OHLC, Order, Tick};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::sleep;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};
use url::Url;

// Mode represents available ticker modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[serde(rename = "ltp")]
    LTP,
    #[serde(rename = "quote")]
    Quote,
    #[serde(rename = "full")]
    Full,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::LTP => write!(f, "ltp"),
            Mode::Quote => write!(f, "quote"),
            Mode::Full => write!(f, "full"),
        }
    }
}

// Command types for internal communication
#[derive(Debug, Clone)]
enum TickerCommand {
    Subscribe(Vec<u32>),
    Unsubscribe(Vec<u32>),
    SetMode(Mode, Vec<u32>),
}

// Segment constants
pub const NSE_CM: u32 = 1;
pub const NSE_FO: u32 = 2;
pub const NSE_CD: u32 = 3;
pub const BSE_CM: u32 = 4;
pub const BSE_FO: u32 = 5;
pub const BSE_CD: u32 = 6;
pub const MCX_FO: u32 = 7;
pub const MCX_SX: u32 = 8;
pub const INDICES: u32 = 9;

// Packet lengths for each mode
const MODE_LTP_LENGTH: usize = 8;
const MODE_QUOTE_INDEX_PACKET_LENGTH: usize = 28;
const MODE_FULL_INDEX_LENGTH: usize = 32;
const MODE_QUOTE_LENGTH: usize = 44;
const MODE_FULL_LENGTH: usize = 184;

// Message types
const MESSAGE_ERROR: &str = "error";
const MESSAGE_ORDER: &str = "order";

// Auto reconnect defaults
const DEFAULT_RECONNECT_MAX_ATTEMPTS: i32 = 300;
const RECONNECT_MIN_DELAY: Duration = Duration::from_millis(5000);
const DEFAULT_RECONNECT_MAX_DELAY: Duration = Duration::from_millis(60000);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(7000);
const CONNECTION_CHECK_INTERVAL: Duration = Duration::from_millis(2000);
const DATA_TIMEOUT_INTERVAL: Duration = Duration::from_millis(5000);

// Default ticker URL
const TICKER_URL: &str = "wss://ws.kite.trade";

#[derive(Debug, Clone)]
pub struct TickerError {
    pub message: String,
}

impl std::fmt::Display for TickerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ticker Error: {}", self.message)
    }
}

impl std::error::Error for TickerError {}

#[derive(Debug, Serialize)]
struct TickerInput {
    #[serde(rename = "a")]
    action_type: String,
    #[serde(rename = "v")]
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct IncomingMessage {
    #[serde(rename = "type")]
    message_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OrderUpdateMessage {
    data: Order,
}

// Event types for the ticker
#[derive(Debug, Clone)]
pub enum TickerEvent {
    Tick(Tick),
    Message(Vec<u8>),
    Connect,
    Close(u16, String),
    Error(String),
    Reconnect(i32, Duration),
    NoReconnect(i32),
    OrderUpdate(Order),
}

// AtomicTime wrapper for safe concurrent access
#[derive(Debug)]
struct AtomicTime {
    timestamp: AtomicU64,
}

impl AtomicTime {
    fn new() -> Self {
        Self {
            timestamp: AtomicU64::new(0),
        }
    }

    fn get(&self) -> SystemTime {
        let ts = self.timestamp.load(Ordering::Relaxed);
        UNIX_EPOCH + Duration::from_secs(ts)
    }

    fn set(&self, time: SystemTime) {
        if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
            self.timestamp.store(duration.as_secs(), Ordering::Relaxed);
        }
    }
}

impl Default for AtomicTime {
    fn default() -> Self {
        Self::new()
    }
}

// Handle for controlling the ticker after it starts
#[derive(Clone)]
pub struct TickerHandle {
    command_sender: mpsc::UnboundedSender<TickerCommand>, // sub, un-sub, set_mode
    event_sender: broadcast::Sender<TickerEvent>,         // tick, error, message.
}

impl TickerHandle {
    pub async fn subscribe(&self, tokens: Vec<u32>) -> Result<(), TickerError> {
        self.command_sender
            .send(TickerCommand::Subscribe(tokens))
            .map_err(|_| TickerError {
                message: "Failed to send subscribe command".to_string(),
            })
    }

    pub async fn unsubscribe(&self, tokens: Vec<u32>) -> Result<(), TickerError> {
        self.command_sender
            .send(TickerCommand::Unsubscribe(tokens))
            .map_err(|_| TickerError {
                message: "Failed to send unsubscribe command".to_string(),
            })
    }

    pub async fn set_mode(&self, mode: Mode, tokens: Vec<u32>) -> Result<(), TickerError> {
        self.command_sender
            .send(TickerCommand::SetMode(mode, tokens))
            .map_err(|_| TickerError {
                message: "Failed to send set_mode command".to_string(),
            })
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<TickerEvent> {
        self.event_sender.subscribe()
    }
}

pub struct Ticker {
    api_key: String,
    access_token: String,
    url: String,
    auto_reconnect: bool,
    reconnect_max_retries: i32,
    reconnect_max_delay: Duration,
    connect_timeout: Duration,
    subscribed_tokens: Arc<RwLock<HashMap<u32, Option<Mode>>>>,
    last_ping_time: Arc<AtomicTime>,
    // channels
    event_sender: broadcast::Sender<TickerEvent>,
    command_receiver: Option<mpsc::UnboundedReceiver<TickerCommand>>,
    command_sender: mpsc::UnboundedSender<TickerCommand>,
}

impl Ticker {
    pub fn new(api_key: String, access_token: String) -> (Self, TickerHandle) {
        let (event_tx, _) = broadcast::channel(1000);
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let ticker = Self {
            api_key,
            access_token,
            url: TICKER_URL.to_string(),
            auto_reconnect: true,
            reconnect_max_retries: DEFAULT_RECONNECT_MAX_ATTEMPTS,
            reconnect_max_delay: DEFAULT_RECONNECT_MAX_DELAY,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            subscribed_tokens: Arc::new(RwLock::new(HashMap::new())),
            last_ping_time: Arc::new(AtomicTime::new()),
            event_sender: event_tx.clone(),
            command_receiver: Some(command_rx),
            command_sender: command_tx.clone(),
        };

        let handle = TickerHandle {
            command_sender: command_tx,
            event_sender: event_tx,
        };

        (ticker, handle)
    }

    pub fn set_root_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = access_token;
    }

    pub fn set_connect_timeout(&mut self, timeout: Duration) {
        self.connect_timeout = timeout;
    }

    pub fn set_auto_reconnect(&mut self, enable: bool) {
        self.auto_reconnect = enable;
    }

    pub fn set_reconnect_max_delay(&mut self, delay: Duration) -> Result<(), TickerError> {
        if delay < RECONNECT_MIN_DELAY {
            return Err(TickerError {
                message: format!(
                    "ReconnectMaxDelay can't be less than {}ms",
                    RECONNECT_MIN_DELAY.as_millis()
                ),
            });
        }
        self.reconnect_max_delay = delay;
        Ok(())
    }

    pub fn set_reconnect_max_retries(&mut self, retries: i32) {
        self.reconnect_max_retries = retries;
    }

    pub async fn serve(mut self) -> Result<(), TickerError> {
        let mut reconnect_attempt = 0;

        loop {
            // If reconnect attempt exceeds max then close the loop
            if reconnect_attempt > self.reconnect_max_retries {
                let _ = self
                    .event_sender
                    .send(TickerEvent::NoReconnect(reconnect_attempt));
                return Err(TickerError {
                    message: "Maximum reconnect attempts reached".to_string(),
                });
            }

            // If its a reconnect then wait exponentially based on reconnect attempt
            if reconnect_attempt > 0 {
                let next_delay = Duration::from_secs(2_u64.pow(reconnect_attempt as u32))
                    .min(self.reconnect_max_delay);

                let _ = self
                    .event_sender
                    .send(TickerEvent::Reconnect(reconnect_attempt, next_delay));
                sleep(next_delay).await;
            }

            // Prepare ticker URL with required params.
            let mut url = Url::parse(&self.url).map_err(|e| TickerError {
                message: format!("Invalid URL: {}", e),
            })?;

            url.query_pairs_mut()
                .append_pair("api_key", &self.api_key)
                .append_pair("access_token", &self.access_token);

            // Connect to WebSocket with timeout
            let connection_future = connect_async(url.as_str());
            match tokio::time::timeout(self.connect_timeout, connection_future).await {
                Ok(Ok((ws_stream, _))) => {
                    // Track if this is a reconnection before resetting counter
                    let is_reconnect = reconnect_attempt > 0;

                    // Reset reconnect attempt on successful connection
                    reconnect_attempt = 0;

                    // Trigger connect event
                    let _ = self.event_sender.send(TickerEvent::Connect);

                    // Set last ping time
                    self.last_ping_time.set(SystemTime::now());

                    // Resubscribe to stored tokens if this is a reconnect
                    if is_reconnect {
                        if let Err(e) = self.resubscribe().await {
                            let _ = self
                                .event_sender
                                .send(TickerEvent::Error(format!("Resubscribe failed: {}", e)));
                        }
                    }

                    // Handle the WebSocket connection
                    if let Err(e) = self.handle_connection(ws_stream).await {
                        let error_msg = e.message.clone();
                        let _ = self
                            .event_sender
                            .send(TickerEvent::Error(error_msg.clone()));

                        if !self.auto_reconnect {
                            return Err(TickerError { message: error_msg });
                        }
                    }
                }
                Ok(Err(e)) => {
                    let error_msg = format!("Connection failed: {}", e);
                    let _ = self
                        .event_sender
                        .send(TickerEvent::Error(error_msg.clone()));

                    if !self.auto_reconnect {
                        return Err(TickerError { message: error_msg });
                    }
                }
                Err(_) => {
                    let error_msg =
                        format!("Connection timed out after {:?}", self.connect_timeout);
                    let _ = self
                        .event_sender
                        .send(TickerEvent::Error(error_msg.clone()));

                    if !self.auto_reconnect {
                        return Err(TickerError { message: error_msg });
                    }
                }
            }

            reconnect_attempt += 1;
        }
    }

    async fn handle_connection(
        &mut self,
        ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> Result<(), TickerError> {
        // Run watcher to check last ping time and reconnect if required
        let reconnect_handler = if self.auto_reconnect {
            let sender_checker = self.event_sender.clone();
            let last_ping_time = self.last_ping_time.clone();

            Some(tokio::spawn(async move {
                loop {
                    sleep(CONNECTION_CHECK_INTERVAL).await;
                    let last_ping = last_ping_time.get();
                    if SystemTime::now()
                        .duration_since(last_ping)
                        .unwrap_or(Duration::ZERO)
                        > DATA_TIMEOUT_INTERVAL
                    {
                        // Connection timeout detected - send error event
                        let _ = sender_checker.send(TickerEvent::Error(
                            "Data timeout: No data received for 5 seconds".to_string(),
                        ));
                        return;
                    }
                }
            }))
        } else {
            None
        };

        // Websocket split
        let (mut write, mut read) = ws_stream.split();

        // Channel for sending messages to WebSocket
        let sender = self.event_sender.clone();

        // Task to handle command processing
        let command_handler = if let Some(command_rx) = self.command_receiver.take() {
            let subscribed_tokens = self.subscribed_tokens.clone();

            Some(tokio::spawn(async move {
                let mut command_rx = command_rx;
                while let Some(command) = command_rx.recv().await {
                    match command {
                        TickerCommand::Subscribe(tokens) => {
                            // Store tokens
                            {
                                let mut subscribed = subscribed_tokens.write().await;
                                for token in &tokens {
                                    subscribed.insert(*token, None);
                                }
                            }

                            let input = TickerInput {
                                action_type: "subscribe".to_string(),
                                value: serde_json::to_value(&tokens).unwrap(),
                            };

                            if let Ok(message) = serde_json::to_string(&input) {
                                if let Err(e) = write.send(Message::Text(message.into())).await {
                                    let _ = sender.send(TickerEvent::Error(format!(
                                        "Failed to send WebSocket message: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        TickerCommand::Unsubscribe(tokens) => {
                            // Remove tokens
                            {
                                let mut subscribed = subscribed_tokens.write().await;
                                for token in &tokens {
                                    subscribed.remove(token);
                                }
                            }

                            let input = TickerInput {
                                action_type: "unsubscribe".to_string(),
                                value: serde_json::to_value(&tokens).unwrap(),
                            };

                            if let Ok(message) = serde_json::to_string(&input) {
                                if let Err(e) = write.send(Message::Text(message.into())).await {
                                    let _ = sender.send(TickerEvent::Error(format!(
                                        "Failed to send WebSocket message: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        TickerCommand::SetMode(mode, tokens) => {
                            // Update mode
                            {
                                let mut subscribed = subscribed_tokens.write().await;
                                for token in &tokens {
                                    subscribed.insert(*token, Some(mode));
                                }
                            }

                            let input = TickerInput {
                                action_type: "mode".to_string(),
                                value: serde_json::to_value(&(mode.to_string(), &tokens)).unwrap(),
                            };

                            if let Ok(message) = serde_json::to_string(&input) {
                                if let Err(e) = write.send(Message::Text(message.into())).await {
                                    let _ = sender.send(TickerEvent::Error(format!(
                                        "Failed to send WebSocket message: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        // Handle incoming messages
        let message_handler = {
            let sender = self.event_sender.clone();
            let last_ping_time = self.last_ping_time.clone();

            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Binary(data)) => {
                            // Update last ping time
                            last_ping_time.set(SystemTime::now());
                            // Trigger message event
                            let _ = sender.send(TickerEvent::Message(data.to_vec()));

                            // Parse binary message and trigger tick events
                            match Ticker::parse_binary(&data) {
                                Ok(ticks) => {
                                    for tick in ticks {
                                        let _ = sender.send(TickerEvent::Tick(tick));
                                    }
                                }
                                Err(e) => {
                                    let _ = sender
                                        .send(TickerEvent::Error(format!("Parse error: {}", e)));
                                }
                            }
                        }
                        Ok(Message::Text(text)) => {
                            // Update last ping time
                            last_ping_time.set(SystemTime::now());

                            // Trigger message event
                            let _ = sender.send(TickerEvent::Message(text.as_bytes().to_vec()));

                            // Process text message
                            Ticker::process_text_message(&text, &sender).await;
                        }
                        Ok(Message::Close(close_frame)) => {
                            // Update last ping time
                            last_ping_time.set(SystemTime::now());

                            let (code, reason) = if let Some(frame) = close_frame {
                                (frame.code.into(), frame.reason.to_string())
                            } else {
                                (1000, "Normal closure".to_string())
                            };
                            let _ = sender.send(TickerEvent::Close(code, reason));
                            break;
                        }
                        Err(e) => {
                            let _ =
                                sender.send(TickerEvent::Error(format!("WebSocket error: {}", e)));
                            break;
                        }
                        _ => {}
                    }
                }
            })
        };

        // Wait for any critical task to complete or fail
        tokio::select! {
            _ = message_handler => {},
            _ = async {
                if let Some(handler) = reconnect_handler {
                    handler.await.ok();
                }
            } => {},
            _ = async {
                if let Some(handler) = command_handler {
                    handler.await.ok();
                }
            } => {},
        }

        Ok(())
    }

    async fn process_text_message(text: &str, sender: &broadcast::Sender<TickerEvent>) {
        if let Ok(msg) = serde_json::from_str::<IncomingMessage>(text) {
            match msg.message_type.as_str() {
                MESSAGE_ERROR => {
                    if let Ok(error_msg) = serde_json::from_value::<String>(msg.data) {
                        let _ = sender.send(TickerEvent::Error(error_msg));
                    }
                }
                MESSAGE_ORDER => {
                    if let Ok(order_msg) = serde_json::from_str::<OrderUpdateMessage>(text) {
                        let _ = sender.send(TickerEvent::OrderUpdate(order_msg.data));
                    }
                }
                _ => {}
            }
        }
    }

    async fn resubscribe(&self) -> Result<(), TickerError> {
        let mut tokens = Vec::new();
        let mut mode_groups: HashMap<Mode, Vec<u32>> = HashMap::new();

        {
            let subscribed = self.subscribed_tokens.read().await;
            for (&token, &mode_opt) in subscribed.iter() {
                tokens.push(token);
                if let Some(mode) = mode_opt {
                    mode_groups.entry(mode).or_insert_with(Vec::new).push(token);
                }
            }
        }

        // Resubscribe to tokens
        if !tokens.is_empty() {
            self.command_sender
                .send(TickerCommand::Subscribe(tokens))
                .map_err(|_| TickerError {
                    message: "Failed to resubscribe".to_string(),
                })?;
        }

        // Set modes for tokens
        for (mode, mode_tokens) in mode_groups {
            if !mode_tokens.is_empty() {
                self.command_sender
                    .send(TickerCommand::SetMode(mode, mode_tokens))
                    .map_err(|_| TickerError {
                        message: "Failed to set mode during resubscribe".to_string(),
                    })?;
            }
        }

        Ok(())
    }

    // Binary parsing methods remain the same
    pub fn parse_binary(data: &[u8]) -> Result<Vec<Tick>, TickerError> {
        let packets = Self::split_packets(data);
        let mut ticks = Vec::new();

        for packet in packets {
            let tick = Self::parse_packet(&packet)?;
            ticks.push(tick);
        }

        Ok(ticks)
    }

    pub fn split_packets(data: &[u8]) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();

        if data.len() < 2 {
            return packets;
        }

        let packet_count = u16::from_be_bytes([data[0], data[1]]) as usize;
        let mut offset = 2;

        for _ in 0..packet_count {
            if offset + 2 > data.len() {
                break;
            }

            let packet_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            if offset + packet_length > data.len() {
                break;
            }

            packets.push(data[offset..offset + packet_length].to_vec());
            offset += packet_length;
        }

        packets
    }

    pub fn parse_packet(data: &[u8]) -> Result<Tick, TickerError> {
        if data.len() < 4 {
            return Err(TickerError {
                message: "Packet too short".to_string(),
            });
        }

        let instrument_token = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let segment = instrument_token & 0xFF;
        let is_index = segment == INDICES;
        let is_tradable = segment != INDICES;

        let mut tick = Tick {
            instrument_token,
            is_tradable,
            is_index,
            ..Default::default()
        };

        match data.len() {
            MODE_LTP_LENGTH => {
                tick.mode = Mode::LTP.to_string();
                tick.last_price = Self::convert_price(segment, Self::read_u32(&data[4..8]));
            }
            MODE_QUOTE_INDEX_PACKET_LENGTH | MODE_FULL_INDEX_LENGTH => {
                tick.mode = if data.len() == MODE_FULL_INDEX_LENGTH {
                    Mode::Full.to_string()
                } else {
                    Mode::Quote.to_string()
                };

                let last_price = Self::convert_price(segment, Self::read_u32(&data[4..8]));
                let close_price = Self::convert_price(segment, Self::read_u32(&data[20..24]));

                tick.last_price = last_price;
                tick.net_change = last_price - close_price;
                tick.ohlc = OHLC {
                    instrument_token: None,
                    high: Self::convert_price(segment, Self::read_u32(&data[8..12])),
                    low: Self::convert_price(segment, Self::read_u32(&data[12..16])),
                    open: Self::convert_price(segment, Self::read_u32(&data[16..20])),
                    close: close_price,
                };

                if data.len() == MODE_FULL_INDEX_LENGTH {
                    tick.timestamp = Time::from_timestamp(Self::read_u32(&data[28..32]) as i64);
                }
            }
            MODE_QUOTE_LENGTH | MODE_FULL_LENGTH => {
                tick.mode = if data.len() == MODE_FULL_LENGTH {
                    Mode::Full.to_string()
                } else {
                    Mode::Quote.to_string()
                };

                let last_price = Self::convert_price(segment, Self::read_u32(&data[4..8]));
                let close_price = Self::convert_price(segment, Self::read_u32(&data[40..44]));

                tick.last_price = last_price;
                tick.last_traded_quantity = Self::read_u32(&data[8..12]);
                tick.average_trade_price =
                    Self::convert_price(segment, Self::read_u32(&data[12..16]));
                tick.volume_traded = Self::read_u32(&data[16..20]);
                tick.total_buy_quantity = Self::read_u32(&data[20..24]);
                tick.total_sell_quantity = Self::read_u32(&data[24..28]);
                tick.net_change = last_price - close_price;

                tick.ohlc = OHLC {
                    instrument_token: None,
                    open: Self::convert_price(segment, Self::read_u32(&data[28..32])),
                    high: Self::convert_price(segment, Self::read_u32(&data[32..36])),
                    low: Self::convert_price(segment, Self::read_u32(&data[36..40])),
                    close: close_price,
                };

                if data.len() == MODE_FULL_LENGTH {
                    tick.last_trade_time =
                        Time::from_timestamp(Self::read_u32(&data[44..48]) as i64);
                    tick.oi = Self::read_u32(&data[48..52]);
                    tick.oi_day_high = Self::read_u32(&data[52..56]);
                    tick.oi_day_low = Self::read_u32(&data[56..60]);
                    tick.timestamp = Time::from_timestamp(Self::read_u32(&data[60..64]) as i64);

                    // Parse depth information
                    let mut buy_pos = 64;
                    let mut sell_pos = 124;

                    for i in 0..5 {
                        if buy_pos + 12 <= data.len() {
                            tick.depth.buy[i] = DepthItem {
                                quantity: Self::read_u32(&data[buy_pos..buy_pos + 4]),
                                price: Self::convert_price(
                                    segment,
                                    Self::read_u32(&data[buy_pos + 4..buy_pos + 8]),
                                ),
                                orders: Self::read_u16(&data[buy_pos + 8..buy_pos + 10]) as u32,
                            };
                            buy_pos += 12;
                        }

                        if sell_pos + 12 <= data.len() {
                            tick.depth.sell[i] = DepthItem {
                                quantity: Self::read_u32(&data[sell_pos..sell_pos + 4]),
                                price: Self::convert_price(
                                    segment,
                                    Self::read_u32(&data[sell_pos + 4..sell_pos + 8]),
                                ),
                                orders: Self::read_u16(&data[sell_pos + 8..sell_pos + 10]) as u32,
                            };
                            sell_pos += 12;
                        }
                    }
                }
            }
            _ => {
                return Err(TickerError {
                    message: format!("Unknown packet length: {}", data.len()),
                });
            }
        }

        Ok(tick)
    }

    fn read_u32(data: &[u8]) -> u32 {
        if data.len() >= 4 {
            u32::from_be_bytes([data[0], data[1], data[2], data[3]])
        } else {
            0
        }
    }

    fn read_u16(data: &[u8]) -> u16 {
        if data.len() >= 2 {
            u16::from_be_bytes([data[0], data[1]])
        } else {
            0
        }
    }

    pub fn convert_price(segment: u32, value: u32) -> f64 {
        let val = value as f64;
        match segment {
            NSE_CD => val / 10_000_000.0,
            BSE_CD => val / 10_000.0,
            _ => val / 100.0,
        }
    }
    pub fn builder(api_key: &str, access_token: &str) -> TickerBuilder {
        TickerBuilder::new(api_key, access_token)
    }
}

pub struct TickerBuilder {
    api_key: String,
    access_token: String,
    url: Option<String>,
    auto_reconnect: Option<bool>,
    reconnect_max_retries: Option<i32>,
    reconnect_max_delay: Option<Duration>,
    connect_timeout: Option<Duration>,
}

impl TickerBuilder {
    pub fn new(api_key: &str, access_token: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            access_token: access_token.to_owned(),
            url: None,
            auto_reconnect: None,
            reconnect_max_retries: None,
            reconnect_max_delay: None,
            connect_timeout: None,
        }
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn auto_reconnect(mut self, enable: bool) -> Self {
        self.auto_reconnect = Some(enable);
        self
    }

    pub fn reconnect_max_retries(mut self, retries: i32) -> Self {
        self.reconnect_max_retries = Some(retries);
        self
    }

    pub fn reconnect_max_delay(mut self, delay: Duration) -> Self {
        self.reconnect_max_delay = Some(delay);
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<(Ticker, TickerHandle), TickerError> {
        let (mut ticker, handle) = Ticker::new(self.api_key, self.access_token);

        if let Some(url) = self.url {
            ticker.set_root_url(url);
        }

        if let Some(auto_reconnect) = self.auto_reconnect {
            ticker.set_auto_reconnect(auto_reconnect);
        }

        if let Some(retries) = self.reconnect_max_retries {
            ticker.set_reconnect_max_retries(retries);
        }

        if let Some(delay) = self.reconnect_max_delay {
            ticker.set_reconnect_max_delay(delay)?;
        }

        if let Some(timeout) = self.connect_timeout {
            ticker.set_connect_timeout(timeout);
        }

        Ok((ticker, handle))
    }
}

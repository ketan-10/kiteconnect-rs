//! Platform compatibility layer for native and WASM targets.
//!
//! This module provides abstractions over platform-specific functionality:
//! - `sleep`: Async sleep that works on both native (tokio) and WASM (gloo-timers)
//! - `spawn`: Task spawning that works on both native (tokio) and WASM (wasm-bindgen-futures)
//! - `timeout`: Async timeout wrapper
//! - `WebSocketStream`: WebSocket abstraction over tokio-tungstenite (native) and gloo-net (WASM)

use async_trait::async_trait;
use std::future::Future;
use web_time::Duration;

// ============================================================================
// Sleep
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

// ============================================================================
// Timeout
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}

#[cfg(target_arch = "wasm32")]
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    use futures_util::future::{select, Either};
    use std::pin::pin;

    let sleep_fut = pin!(sleep(duration));
    let task_fut = pin!(future);

    match select(task_fut, sleep_fut).await {
        Either::Left((result, _)) => Ok(result),
        Either::Right((_, _)) => Err(TimeoutError),
    }
}

#[derive(Debug, Clone)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

// ============================================================================
// Spawn
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F) -> TaskHandle
where
    F: Future<Output = ()> + Send + 'static,
{
    let handle = tokio::spawn(future);
    TaskHandle {
        inner: Some(TaskHandleInner::Native(handle)),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F) -> TaskHandle
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
    TaskHandle { inner: None }
}

pub struct TaskHandle {
    #[cfg(not(target_arch = "wasm32"))]
    inner: Option<TaskHandleInner>,
    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    inner: Option<()>,
}

#[cfg(not(target_arch = "wasm32"))]
enum TaskHandleInner {
    Native(tokio::task::JoinHandle<()>),
}

impl TaskHandle {
    pub fn abort(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(TaskHandleInner::Native(ref handle)) = self.inner {
            handle.abort();
        }
        // WASM: spawn_local tasks cannot be aborted, this is a no-op
    }
}

// ============================================================================
// WebSocket
// ============================================================================

#[derive(Debug, Clone)]
pub struct WsError(pub String);

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebSocket error: {}", self.0)
    }
}

impl std::error::Error for WsError {}

#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<(u16, String)>),
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait WebSocketStream: Send {
    async fn send_text(&mut self, msg: String) -> Result<(), WsError>;
    async fn send_binary(&mut self, msg: Vec<u8>) -> Result<(), WsError>;
    async fn recv(&mut self) -> Option<Result<WsMessage, WsError>>;
    async fn close(&mut self) -> Result<(), WsError>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait WebSocketStream {
    async fn send_text(&mut self, msg: String) -> Result<(), WsError>;
    async fn send_binary(&mut self, msg: Vec<u8>) -> Result<(), WsError>;
    async fn recv(&mut self) -> Option<Result<WsMessage, WsError>>;
    async fn close(&mut self) -> Result<(), WsError>;
}

// ============================================================================
// Native WebSocket Implementation (tokio-tungstenite)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod native_ws {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpStream;
    use tokio_tungstenite::{
        connect_async, tungstenite::Message, MaybeTlsStream,
        WebSocketStream as TungsteniteWs,
    };

    pub struct NativeWebSocket {
        inner: TungsteniteWs<MaybeTlsStream<TcpStream>>,
    }

    impl NativeWebSocket {
        pub async fn connect(url: &str) -> Result<Self, WsError> {
            let (ws_stream, _) = connect_async(url)
                .await
                .map_err(|e| WsError(e.to_string()))?;
            Ok(Self { inner: ws_stream })
        }
    }

    #[async_trait]
    impl WebSocketStream for NativeWebSocket {
        async fn send_text(&mut self, msg: String) -> Result<(), WsError> {
            self.inner
                .send(Message::Text(msg.into()))
                .await
                .map_err(|e| WsError(e.to_string()))
        }

        async fn send_binary(&mut self, msg: Vec<u8>) -> Result<(), WsError> {
            self.inner
                .send(Message::Binary(msg.into()))
                .await
                .map_err(|e| WsError(e.to_string()))
        }

        async fn recv(&mut self) -> Option<Result<WsMessage, WsError>> {
            match self.inner.next().await {
                Some(Ok(Message::Text(text))) => Some(Ok(WsMessage::Text(text.to_string()))),
                Some(Ok(Message::Binary(data))) => Some(Ok(WsMessage::Binary(data.to_vec()))),
                Some(Ok(Message::Close(frame))) => {
                    let close_info = frame.map(|f| (f.code.into(), f.reason.to_string()));
                    Some(Ok(WsMessage::Close(close_info)))
                }
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                    // Skip ping/pong, get next message
                    Box::pin(self.recv()).await
                }
                Some(Ok(Message::Frame(_))) => {
                    // Skip raw frames, get next message
                    Box::pin(self.recv()).await
                }
                Some(Err(e)) => Some(Err(WsError(e.to_string()))),
                None => None,
            }
        }

        async fn close(&mut self) -> Result<(), WsError> {
            self.inner
                .close(None)
                .await
                .map_err(|e| WsError(e.to_string()))
        }
    }
}

// ============================================================================
// WASM WebSocket Implementation (gloo-net)
// ============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm_ws {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use gloo_net::websocket::{futures::WebSocket, Message};

    pub struct WasmWebSocket {
        inner: Option<WebSocket>,
    }

    impl WasmWebSocket {
        pub fn connect(url: &str) -> Result<Self, WsError> {
            let ws = WebSocket::open(url).map_err(|e| WsError(e.to_string()))?;
            Ok(Self { inner: Some(ws) })
        }
    }

    #[async_trait(?Send)]
    impl WebSocketStream for WasmWebSocket {
        async fn send_text(&mut self, msg: String) -> Result<(), WsError> {
            if let Some(ref mut ws) = self.inner {
                ws.send(Message::Text(msg))
                    .await
                    .map_err(|e| WsError(e.to_string()))
            } else {
                Err(WsError("WebSocket is closed".to_string()))
            }
        }

        async fn send_binary(&mut self, msg: Vec<u8>) -> Result<(), WsError> {
            if let Some(ref mut ws) = self.inner {
                ws.send(Message::Bytes(msg))
                    .await
                    .map_err(|e| WsError(e.to_string()))
            } else {
                Err(WsError("WebSocket is closed".to_string()))
            }
        }

        async fn recv(&mut self) -> Option<Result<WsMessage, WsError>> {
            if let Some(ref mut ws) = self.inner {
                match ws.next().await {
                    Some(Ok(Message::Text(text))) => Some(Ok(WsMessage::Text(text))),
                    Some(Ok(Message::Bytes(data))) => Some(Ok(WsMessage::Binary(data))),
                    Some(Err(e)) => Some(Err(WsError(e.to_string()))),
                    None => None,
                }
            } else {
                None
            }
        }

        async fn close(&mut self) -> Result<(), WsError> {
            if let Some(ws) = self.inner.take() {
                ws.close(None, None)
                    .map_err(|e| WsError(format!("{:?}", e)))
            } else {
                Ok(())
            }
        }
    }
}

// ============================================================================
// Public WebSocket connect function
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub async fn connect_ws(url: &str) -> Result<Box<dyn WebSocketStream>, WsError> {
    let ws = native_ws::NativeWebSocket::connect(url).await?;
    Ok(Box::new(ws))
}

#[cfg(target_arch = "wasm32")]
pub async fn connect_ws(url: &str) -> Result<Box<dyn WebSocketStream>, WsError> {
    let ws = wasm_ws::WasmWebSocket::connect(url)?;
    Ok(Box::new(ws))
}

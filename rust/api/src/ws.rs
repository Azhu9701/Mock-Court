use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::SinkExt;
use futures_util::StreamExt;

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path((session_id, channel)): Path<(String, String)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state, session_id, channel))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>, session_id: String, channel: String) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    state.engine.ws_manager().subscribe(&session_id, &channel, tx);
    tracing::info!("WS connected: session={} channel={}", session_id, channel);

    let mut send_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if ws_tx
                .send(Message::Text(json.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => {},
        _ = &mut recv_task => {},
    }

    state.engine.ws_manager().unsubscribe(&session_id, &channel);
    tracing::info!("WS disconnected: session={} channel={}", session_id, channel);
}

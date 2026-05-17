use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, WebSocketUpgrade};
use axum::response::IntoResponse;
use foundation::SessionStatus;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::sync::broadcast;

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
    let (tx, mut rx) = tokio::sync::mpsc::channel::<possession::WsEvent>(256);

    let existed = state.engine.ws_manager().subscribe(&session_id, &channel, tx);
    tracing::info!("WS connected: session={} channel={} existed={}", session_id, channel, existed);

    if !existed {
        if let Ok(detail) = state.archive.get_session_detail(&session_id).await {
            if detail.session.status == SessionStatus::Completed {
                let _ = state.engine.ws_manager().broadcast_system(&session_id, &possession::WsEvent {
                    event_type: possession::WsEventType::SessionComplete,
                    payload: String::new(),
                    reasoning_content: None,
                    soul_name: None,
                    seq: 0,
                });
                tracing::info!("Sent SessionComplete for completed session {}", session_id);
            }
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if ws_tx
                        .send(Message::Text(json.into()))
                        .await
                        .is_err()
                    {
                        let _ = ws_tx
                            .send(Message::Close(None))
                            .await;
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize WsEvent: {} — event_type={:?}", e, event.event_type);
                }
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

/// WS `/ws/souls/auto-create/:task_id` — push 收魂+炼化 progress events.
pub async fn auto_create_ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_auto_create_ws(socket, state, task_id))
}

async fn handle_auto_create_ws(socket: WebSocket, state: Arc<AppState>, task_id: String) {
    let tx = match state.auto_create_tasks.get(&task_id) {
        Some(tx) => tx,
        None => {
            tracing::warn!("auto_create_ws: task {} not found", task_id);
            return;
        }
    };

    let mut rx = tx.subscribe();
    let (mut ws_tx, mut ws_rx) = socket.split();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(evt) => {
                        let is_terminal = evt.phase == "done" || evt.phase == "error";
                        if let Ok(json) = serde_json::to_string(&evt) {
                            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        if is_terminal { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = ws_rx.next() => {} // client closed
        }
    }

    tracing::info!("auto_create_ws disconnected: task={}", task_id);
}

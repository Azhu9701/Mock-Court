use dashmap::DashMap;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::WsEvent;

#[derive(Debug, Clone, Default)]
pub struct WsSessionManager {
    sessions: std::sync::Arc<DashMap<String, WsSessionState>>,
}

#[derive(Debug, Clone, Default)]
struct WsSessionState {
    soul_channels: HashMap<String, Vec<mpsc::Sender<WsEvent>>>,
    system_channel: Vec<mpsc::Sender<WsEvent>>,
}

impl WsSessionManager {
    pub fn new() -> Self {
        WsSessionManager {
            sessions: std::sync::Arc::new(DashMap::new()),
        }
    }

    pub fn create_session(&self, session_id: &str) {
        self.sessions.entry(session_id.to_string()).or_insert_with(|| WsSessionState {
            soul_channels: HashMap::new(),
            system_channel: Vec::new(),
        });
    }

    pub fn subscribe_soul(
        &self,
        session_id: &str,
        soul_name: &str,
        tx: mpsc::Sender<WsEvent>,
    ) {
        self.sessions.entry(session_id.to_string()).and_modify(|state| {
            state
                .soul_channels
                .entry(soul_name.to_string())
                .or_default()
                .push(tx);
        });
    }

    pub fn broadcast_soul(
        &self,
        session_id: &str,
        soul_name: &str,
        event: &WsEvent,
    ) {
        if let Some(state) = self.sessions.get(session_id) {
            if let Some(senders) = state.soul_channels.get(soul_name) {
                for tx in senders {
                    let _ = tx.try_send(event.clone());
                }
            }
            for tx in &state.system_channel {
                let _ = tx.try_send(event.clone());
            }
        }
    }

    pub fn broadcast_system(&self, session_id: &str, event: &WsEvent) {
        if let Some(state) = self.sessions.get(session_id) {
            tracing::info!("Broadcasting to {} system subscribers for session {}", state.system_channel.len(), session_id);
            for tx in &state.system_channel {
                if let Err(e) = tx.try_send(event.clone()) {
                    tracing::warn!("Failed to send event to subscriber: {}", e);
                }
            }
        } else {
            tracing::warn!("No session found for broadcasting: {}", session_id);
        }
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn handle_reconnect(
        &self,
        session_id: &str,
        new_system_tx: mpsc::Sender<WsEvent>,
    ) {
        self.sessions.entry(session_id.to_string()).and_modify(|state| {
            state.system_channel.push(new_system_tx);
        });
    }

    pub fn subscribe(
        &self,
        session_id: &str,
        channel: &str,
        tx: mpsc::Sender<WsEvent>,
    ) {
        tracing::info!("New subscription: session={}, channel={}", session_id, channel);
        self.sessions.entry(session_id.to_string()).and_modify(|state| {
            match channel {
                "main" => {
                    state.system_channel.push(tx);
                    tracing::info!("Added to system channel, now {} subscribers", state.system_channel.len());
                }
                _ => {
                    state
                        .soul_channels
                        .entry(channel.to_string())
                        .or_default()
                        .push(tx);
                }
            }
        });
    }

    pub fn unsubscribe(&self, session_id: &str, channel: &str) {
        self.sessions.entry(session_id.to_string()).and_modify(|state| {
            match channel {
                "main" => state.system_channel.clear(),
                _ => {
                    state.soul_channels.remove(channel);
                }
            }
        });
    }

    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }
}

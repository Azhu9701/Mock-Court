use std::collections::HashMap;
use std::sync::RwLock;

use tokio::sync::mpsc::UnboundedSender;

use crate::WsEvent;

#[derive(Debug, Clone, Default)]
pub struct WsSessionManager {
    sessions: std::sync::Arc<RwLock<HashMap<String, WsSessionState>>>,
}

#[derive(Debug, Clone, Default)]
struct WsSessionState {
    soul_channels: HashMap<String, Vec<UnboundedSender<WsEvent>>>,
    system_channel: Vec<UnboundedSender<WsEvent>>,
}

impl WsSessionManager {
    pub fn new() -> Self {
        WsSessionManager {
            sessions: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().expect("ws sessions lock poisoned");
        sessions.entry(session_id.to_string()).or_insert_with(|| WsSessionState {
            soul_channels: HashMap::new(),
            system_channel: Vec::new(),
        });
    }

    pub fn subscribe_soul(
        &self,
        session_id: &str,
        soul_name: &str,
        tx: UnboundedSender<WsEvent>,
    ) {
        let mut sessions = self.sessions.write().expect("ws sessions lock poisoned");
        if let Some(state) = sessions.get_mut(session_id) {
            state
                .soul_channels
                .entry(soul_name.to_string())
                .or_default()
                .push(tx);
        }
    }

    pub fn broadcast_soul(
        &self,
        session_id: &str,
        soul_name: &str,
        event: &WsEvent,
    ) {
        let sessions = self.sessions.read().expect("ws sessions lock poisoned");
        if let Some(state) = sessions.get(session_id) {
            if let Some(senders) = state.soul_channels.get(soul_name) {
                for tx in senders {
                    let _ = tx.send(event.clone());
                }
            }
        }
    }

    pub fn broadcast_system(&self, session_id: &str, event: &WsEvent) {
        let sessions = self.sessions.read().expect("ws sessions lock poisoned");
        if let Some(state) = sessions.get(session_id) {
            tracing::info!("Broadcasting to {} system subscribers for session {}", state.system_channel.len(), session_id);
            for tx in &state.system_channel {
                if let Err(e) = tx.send(event.clone()) {
                    tracing::warn!("Failed to send event to subscriber: {}", e);
                }
            }
        } else {
            tracing::warn!("No session found for broadcasting: {}", session_id);
        }
    }

    pub fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().expect("ws sessions lock poisoned");
        sessions.remove(session_id);
    }

    pub fn handle_reconnect(
        &self,
        session_id: &str,
        new_system_tx: UnboundedSender<WsEvent>,
    ) {
        let mut sessions = self.sessions.write().expect("ws sessions lock poisoned");
        if let Some(state) = sessions.get_mut(session_id) {
            state.system_channel.push(new_system_tx);
        }
    }

    pub fn subscribe(
        &self,
        session_id: &str,
        channel: &str,
        tx: UnboundedSender<WsEvent>,
    ) {
        let mut sessions = self.sessions.write().expect("ws sessions lock poisoned");
        tracing::info!("New subscription: session={}, channel={}", session_id, channel);
        match channel {
            "main" => {
                if let Some(state) = sessions.get_mut(session_id) {
                    state.system_channel.push(tx);
                    tracing::info!("Added to system channel, now {} subscribers", state.system_channel.len());
                }
            }
            _ => {
                if let Some(state) = sessions.get_mut(session_id) {
                    state
                        .soul_channels
                        .entry(channel.to_string())
                        .or_default()
                        .push(tx);
                }
            }
        }
    }

    pub fn unsubscribe(&self, session_id: &str, channel: &str) {
        if let Ok(mut sessions) = self.sessions.write() {
            if let Some(state) = sessions.get_mut(session_id) {
                match channel {
                    "main" => state.system_channel.clear(),
                    _ => {
                        state.soul_channels.remove(channel);
                    }
                }
            }
        }
    }

    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions
            .read()
            .map(|s| s.contains_key(session_id))
            .unwrap_or(false)
    }
}

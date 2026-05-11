use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc;

use crate::WsEvent;

const MAX_BUFFERED_EVENTS: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct WsSessionManager {
    sessions: std::sync::Arc<DashMap<String, WsSessionState>>,
}

#[derive(Debug, Clone, Default)]
struct WsSessionState {
    soul_channels: HashMap<String, Vec<mpsc::Sender<WsEvent>>>,
    system_channel: Vec<mpsc::Sender<WsEvent>>,
    event_buffer: VecDeque<WsEvent>,
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
            event_buffer: VecDeque::with_capacity(MAX_BUFFERED_EVENTS),
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
        }
        self.broadcast_system(session_id, event);
    }

    pub fn broadcast_system(&self, session_id: &str, event: &WsEvent) {
        if let Some(mut state) = self.sessions.get_mut(session_id) {
            state.event_buffer.push_back(event.clone());
            while state.event_buffer.len() > MAX_BUFFERED_EVENTS {
                state.event_buffer.pop_front();
            }
            for tx in &state.system_channel {
                let _ = tx.try_send(event.clone());
            }
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
            for event in &state.event_buffer {
                let _ = new_system_tx.try_send(event.clone());
            }
            state.system_channel.push(new_system_tx);
        });
    }

    pub fn subscribe(
        &self,
        session_id: &str,
        channel: &str,
        tx: mpsc::Sender<WsEvent>,
    ) -> bool {
        tracing::info!("New subscription: session={}, channel={}", session_id, channel);
        let existed = self.sessions.contains_key(session_id);
        let tx2 = tx.clone();
        if existed {
            self.sessions.entry(session_id.to_string()).and_modify(|state| {
                match channel {
                    "main" => {
                        for event in &state.event_buffer {
                            let _ = tx.try_send(event.clone());
                        }
                        state.system_channel.push(tx);
                        tracing::info!("Added to system channel, now {} subscribers (replayed {} buffered events)", state.system_channel.len(), state.event_buffer.len());
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
        } else {
            let sid = session_id.to_string();
            if channel == "main" {
                self.sessions.insert(sid, WsSessionState {
                    soul_channels: HashMap::new(),
                    system_channel: vec![tx2],
                    event_buffer: VecDeque::with_capacity(MAX_BUFFERED_EVENTS),
                });
            } else {
                let mut state = WsSessionState::default();
                state.soul_channels.entry(channel.to_string()).or_default().push(tx2);
                self.sessions.insert(sid, state);
            }
            tracing::info!("Created new session entry for {} (session not in memory)", session_id);
        }
        existed
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

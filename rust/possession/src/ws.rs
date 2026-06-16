use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;

use crate::WsEvent;

const MAX_BUFFERED_EVENTS: usize = 200;
/// Sessions inactive for more than 1 hour are eligible for cleanup
const SESSION_TTL_SECS: u64 = 3600;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Default)]
pub struct WsSessionManager {
    sessions: std::sync::Arc<DashMap<String, WsSessionState>>,
}

#[derive(Debug)]
struct WsSessionState {
    soul_channels: HashMap<String, Vec<mpsc::Sender<WsEvent>>>,
    system_channel: Vec<mpsc::Sender<WsEvent>>,
    event_buffer: VecDeque<WsEvent>,
    last_activity: AtomicU64,
}

impl Clone for WsSessionState {
    fn clone(&self) -> Self {
        Self {
            soul_channels: self.soul_channels.clone(),
            system_channel: self.system_channel.clone(),
            event_buffer: self.event_buffer.clone(),
            last_activity: AtomicU64::new(self.last_activity.load(Ordering::Relaxed)),
        }
    }
}

impl Default for WsSessionState {
    fn default() -> Self {
        Self {
            soul_channels: HashMap::new(),
            system_channel: Vec::new(),
            event_buffer: VecDeque::with_capacity(MAX_BUFFERED_EVENTS),
            last_activity: AtomicU64::new(now_secs()),
        }
    }
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
            last_activity: AtomicU64::new(now_secs()),
        });
    }

    fn touch(&self, session_id: &str) {
        if let Some(state) = self.sessions.get(session_id) {
            state.last_activity.store(now_secs(), Ordering::Relaxed);
        }
    }

    pub fn subscribe_soul(
        &self,
        session_id: &str,
        soul_name: &str,
        tx: mpsc::Sender<WsEvent>,
    ) {
        self.touch(session_id);
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
        self.touch(session_id);
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
        self.touch(session_id);
        if let Some(mut state) = self.sessions.get_mut(session_id) {
            let event_type: &str = &serde_json::to_string(&event.event_type).unwrap_or_default();
            let is_stream_chunk = event_type == "\"soul_token\"" || event_type == "\"synthesis_chunk\"";
            if !is_stream_chunk {
                state.event_buffer.push_back(event.clone());
                while state.event_buffer.len() > MAX_BUFFERED_EVENTS {
                    state.event_buffer.pop_front();
                }
            }
            for tx in &state.system_channel {
                let _ = tx.try_send(event.clone());
            }
        }
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Remove sessions whose all channels are empty and TTL has expired
    pub fn cleanup_stale_sessions(&self) {
        let now = now_secs();
        let before = self.sessions.len();
        self.sessions.retain(|_id, state| {
            let has_subscribers = !state.system_channel.is_empty()
                || state.soul_channels.values().any(|v| !v.is_empty());
            let alive = has_subscribers
                || now.saturating_sub(state.last_activity.load(Ordering::Relaxed)) < SESSION_TTL_SECS;
            alive
        });
        let removed = before.saturating_sub(self.sessions.len());
        if removed > 0 {
            tracing::info!("Cleaned up {} stale WS sessions ({} remaining)", removed, self.sessions.len());
        }
    }

    pub fn handle_reconnect(
        &self,
        session_id: &str,
        new_system_tx: mpsc::Sender<WsEvent>,
    ) {
        self.touch(session_id);
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
        self.touch(session_id);
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
                    last_activity: AtomicU64::new(now_secs()),
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
        let should_remove = if let Some(mut state) = self.sessions.get_mut(session_id) {
            match channel {
                "main" => state.system_channel.clear(),
                _ => {
                    state.soul_channels.remove(channel);
                }
            }
            // Remove session if all channels are empty (no active subscribers)
            state.system_channel.is_empty() && state.soul_channels.values().all(|v| v.is_empty())
        } else {
            false
        };
        if should_remove {
            self.sessions.remove(session_id);
            tracing::info!("Removed empty WS session: {}", session_id);
        }
    }

    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

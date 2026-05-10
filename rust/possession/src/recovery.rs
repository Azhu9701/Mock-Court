use foundation::{SessionFilter, SessionStatus, Storage};

pub struct RecoveryManager;

impl RecoveryManager {
    pub async fn recover_active_sessions(store: &dyn Storage) -> foundation::Result<Vec<String>> {
        let sessions = store
            .list_sessions(&SessionFilter {
                status: Some(SessionStatus::Active),
                ..Default::default()
            })
            .await?;

        let ids: Vec<String> = sessions.into_iter().map(|s| s.id).collect();
        tracing::info!("Found {} active sessions to recover", ids.len());

        for id in &ids {
            let mut session = store.get_session(id).await?;
            session.status = SessionStatus::Inconsistent;
            session.updated_at = chrono::Utc::now();
            store.update_session(&session).await?;
            tracing::info!("Marked session {} as inconsistent", id);
        }

        Ok(ids)
    }
}

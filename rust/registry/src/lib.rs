mod ismism;
mod search;
pub mod fulltext_search;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use foundation::{
    FoundationError, IsmismFilter, IsmismStats, Result, SoulListEntry, SoulMatch, SoulProfile,
    Storage,
};

use crate::ismism::compute_distribution;
use crate::search::{build_inverted_index, fulltext_search, nearest_search};

pub struct SoulRegistry {
    store: Arc<dyn Storage>,
    souls: RwLock<HashMap<String, SoulProfile>>,
    inverted_index: RwLock<HashMap<String, Vec<String>>>,
}

impl SoulRegistry {
    pub async fn new(store: Arc<dyn Storage>) -> Result<Self> {
        let registry = SoulRegistry {
            store,
            souls: RwLock::new(HashMap::new()),
            inverted_index: RwLock::new(HashMap::new()),
        };
        registry.reload().await?;
        Ok(registry)
    }

    pub async fn reload(&self) -> Result<()> {
        let names = self.store.list_soul_names().await?;
        let mut souls = HashMap::new();

        // Parallel load all souls
        let mut set = tokio::task::JoinSet::new();
        for name in &names {
            let name = name.clone();
            let store = self.store.clone();
            set.spawn(async move {
                match store.read_soul(&name).await {
                    Ok(profile) => Some((name, profile)),
                    Err(e) => { tracing::warn!("Skipping soul {}: {}", name, e); None }
                }
            });
        }
        while let Some(r) = set.join_next().await {
            if let Ok(Some((name, profile))) = r {
                souls.insert(name, profile);
            }
        }

        let index = build_inverted_index(&souls);

        let mut souls_lock = self
            .souls
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        *souls_lock = souls;

        let mut index_lock = self
            .inverted_index
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        *index_lock = index;

        Ok(())
    }

    pub fn list_souls(&self, filter: &IsmismFilter) -> Result<Vec<SoulListEntry>> {
        let souls = self
            .souls
            .read()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;

        if let Some(ref nearest) = filter.nearest {
            let results = nearest_search(&nearest.target, &souls, nearest.limit);
            let entries: Vec<SoulListEntry> = results
                .into_iter()
                .map(|m| m.entry)
                .collect();
            return Ok(entries);
        }

        let mut entries: Vec<SoulListEntry> = souls
            .values()
            .map(SoulListEntry::from)
            .collect();

        entries.sort_by_key(|e| std::cmp::Reverse(e.summon_count));
        Ok(entries)
    }

    pub fn get_soul(&self, name: &str) -> Result<SoulProfile> {
        let souls = self
            .souls
            .read()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        souls
            .get(name)
            .cloned()
            .ok_or_else(|| FoundationError::SoulNotFound(name.to_string()))
    }

    pub fn search_souls(&self, query: &str) -> Result<Vec<SoulMatch>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let souls = self
            .souls
            .read()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        let index = self
            .inverted_index
            .read()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        Ok(fulltext_search(query, &souls, &index))
    }

    pub fn get_ismism_distribution(&self) -> Result<IsmismStats> {
        let souls = self
            .souls
            .read()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        let entries: Vec<SoulListEntry> = souls.values().map(SoulListEntry::from).collect();
        Ok(compute_distribution(&entries))
    }

    // CRUD

    pub async fn create_soul(&self, profile: SoulProfile) -> Result<()> {
        let name = profile.name.clone();
        self.store.write_soul(&profile).await?;

        let mut souls = self
            .souls
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        souls.insert(name, profile);
        self.rebuild_index(&souls)?;
        Ok(())
    }

    pub async fn update_soul(&self, profile: SoulProfile) -> Result<()> {
        let name = profile.name.clone();
        self.store.write_soul(&profile).await?;

        let mut souls = self
            .souls
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        souls.insert(name, profile);
        self.rebuild_index(&souls)?;
        Ok(())
    }

    pub async fn delete_soul(&self, name: &str) -> Result<()> {
        self.store.delete_soul(name).await?;

        let mut souls = self
            .souls
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        souls.remove(name);
        self.rebuild_index(&souls)?;
        Ok(())
    }

    fn rebuild_index(&self, souls: &HashMap<String, SoulProfile>) -> Result<()> {
        let index = build_inverted_index(souls);
        let mut index_lock = self
            .inverted_index
            .write()
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        *index_lock = index;
        Ok(())
    }
}

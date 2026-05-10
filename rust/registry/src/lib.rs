mod ismism;
mod search;
pub mod fulltext_search;

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;

use foundation::{
    FoundationError, IsmismFilter, IsmismStats, Result, SoulListEntry, SoulMatch, SoulProfile,
    Storage,
};

use crate::ismism::compute_distribution;
use crate::search::{build_inverted_index, fulltext_search, nearest_search};

pub struct SoulRegistry {
    store: Arc<dyn Storage>,
    souls: DashMap<String, SoulProfile>,
    inverted_index: DashMap<String, Vec<String>>,
}

impl SoulRegistry {
    pub async fn new(store: Arc<dyn Storage>) -> Result<Self> {
        let registry = SoulRegistry {
            store,
            souls: DashMap::new(),
            inverted_index: DashMap::new(),
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

        self.souls.clear();
        for (k, v) in souls {
            self.souls.insert(k, v);
        }
        self.inverted_index.clear();
        for (k, v) in index {
            self.inverted_index.insert(k, v);
        }

        Ok(())
    }

    pub fn list_souls(&self, filter: &IsmismFilter) -> Result<Vec<SoulListEntry>> {
        if let Some(ref nearest) = filter.nearest {
            let souls: HashMap<String, SoulProfile> = self
                .souls
                .iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();
            let results = nearest_search(&nearest.target, &souls, nearest.limit);
            let entries: Vec<SoulListEntry> = results
                .into_iter()
                .map(|m| m.entry)
                .collect();
            return Ok(entries);
        }

        let mut entries: Vec<SoulListEntry> = self
            .souls
            .iter()
            .map(|r| SoulListEntry::from(r.value()))
            .collect();

        entries.sort_by_key(|e| std::cmp::Reverse(e.summon_count));
        Ok(entries)
    }

    pub fn get_soul(&self, name: &str) -> Result<SoulProfile> {
        self.souls
            .get(name)
            .map(|r| r.clone())
            .ok_or_else(|| FoundationError::SoulNotFound(name.to_string()))
    }

    pub fn search_souls(&self, query: &str) -> Result<Vec<SoulMatch>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let souls: HashMap<String, SoulProfile> = self
            .souls
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        let index: HashMap<String, Vec<String>> = self
            .inverted_index
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        Ok(fulltext_search(query, &souls, &index))
    }

    pub fn get_ismism_distribution(&self) -> Result<IsmismStats> {
        let entries: Vec<SoulListEntry> = self
            .souls
            .iter()
            .map(|r| SoulListEntry::from(r.value()))
            .collect();
        Ok(compute_distribution(&entries))
    }

    // CRUD

    pub async fn create_soul(&self, profile: SoulProfile) -> Result<()> {
        let name = profile.name.clone();
        self.store.write_soul(&profile).await?;

        self.souls.insert(name, profile);
        let souls_map: HashMap<String, SoulProfile> = self
            .souls
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        self.rebuild_index(&souls_map)?;
        Ok(())
    }

    pub async fn update_soul(&self, profile: SoulProfile) -> Result<()> {
        let name = profile.name.clone();
        self.store.write_soul(&profile).await?;

        self.souls.insert(name, profile);
        let souls_map: HashMap<String, SoulProfile> = self
            .souls
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        self.rebuild_index(&souls_map)?;
        Ok(())
    }

    pub async fn delete_soul(&self, name: &str) -> Result<()> {
        self.store.delete_soul(name).await?;

        self.souls.remove(name);
        let souls_map: HashMap<String, SoulProfile> = self
            .souls
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        self.rebuild_index(&souls_map)?;
        Ok(())
    }

    fn rebuild_index(&self, souls: &HashMap<String, SoulProfile>) -> Result<()> {
        let index = build_inverted_index(souls);
        self.inverted_index.clear();
        for (k, v) in index {
            self.inverted_index.insert(k, v);
        }
        Ok(())
    }
}

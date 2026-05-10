use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{FoundationError, Result};
use crate::models::*;

pub struct FileStore {
    souls_dir: PathBuf,
    archive_dir: PathBuf,
    registry_path: PathBuf,
    call_records_path: PathBuf,
    registry_cache: std::sync::RwLock<HashMap<String, RegistryEntry>>,
}

impl FileStore {
    pub fn new(souls_dir: PathBuf, archive_dir: PathBuf, registry_path: PathBuf, call_records_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&souls_dir)?;
        std::fs::create_dir_all(&archive_dir)?;
        let store = FileStore {
            souls_dir,
            archive_dir,
            registry_path,
            call_records_path,
            registry_cache: std::sync::RwLock::new(HashMap::new()),
        };
        store.reload_registry()?;
        Ok(store)
    }

    // Soul operations with YAML frontmatter + Markdown body
    pub fn read_soul(&self, name: &str) -> Result<SoulProfile> {
        let path = self.soul_path(name);
        if !path.exists() {
            return Err(FoundationError::SoulNotFound(name.to_string()));
        }
        let content = std::fs::read_to_string(&path)?;
        Self::parse_soul_md(&content)
    }

    pub fn write_soul(&self, profile: &SoulProfile) -> Result<()> {
        let path = self.soul_path(&profile.name);
        let content = Self::serialize_soul_md(profile)?;
        self.atomic_write(&path, &content)?;
        Ok(())
    }

    pub fn delete_soul(&self, name: &str) -> Result<()> {
        let path = self.soul_path(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn list_soul_names(&self) -> Result<Vec<String>> {
        let mut names = vec![];
        if self.souls_dir.exists() {
            for entry in std::fs::read_dir(&self.souls_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
        Ok(names)
    }

    // Registry operations
    pub fn read_registry_raw(&self) -> Result<Registry> {
        if !self.registry_path.exists() {
            return Ok(Registry { souls: HashMap::new() });
        }
        let content = std::fs::read_to_string(&self.registry_path)?;
        let registry: Registry = serde_yaml::from_str(&content)?;
        Ok(registry)
    }

    pub fn write_registry_raw(&self, registry: &Registry) -> Result<()> {
        let content = serde_yaml::to_string(registry)?;
        self.atomic_write(&self.registry_path, &content)?;
        let mut cache = self.registry_cache.write().map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        *cache = registry.souls.clone();
        Ok(())
    }

    pub fn reload_registry(&self) -> Result<()> {
        let registry = self.read_registry_raw()?;
        let mut cache = self.registry_cache.write().map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        *cache = registry.souls;
        Ok(())
    }

    pub fn get_registry_entry(&self, name: &str) -> Option<RegistryEntry> {
        self.registry_cache.read().ok()?.get(name).cloned()
    }

    pub fn list_registry_entries(&self) -> Result<Vec<(String, RegistryEntry)>> {
        let cache = self.registry_cache.read().map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        Ok(cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    pub fn registry_entry_count(&self) -> usize {
        self.registry_cache.read().map(|c| c.len()).unwrap_or(0)
    }

    // Archive operations
    pub fn archive_output(&self, session_id: &str, filename: &str, content: &str) -> Result<String> {
        let now = Utc::now();
        let dir = self.archive_dir
            .join(now.format("%Y").to_string())
            .join(now.format("%m").to_string())
            .join(now.format("%d").to_string())
            .join(session_id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(filename);
        self.atomic_write(&path, content)?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn read_archive_path(&self, path: &str) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    // Call Records YAML
    pub fn read_call_records_yaml(&self) -> Result<Vec<CallRecord>> {
        if !self.call_records_path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&self.call_records_path)?;
        let records: Vec<CallRecord> = serde_yaml::from_str(&content)?;
        Ok(records)
    }

    pub fn append_call_record_yaml(&self, record: &CallRecord) -> Result<()> {
        let mut records = self.read_call_records_yaml().unwrap_or_default();
        records.push(record.clone());
        let content = serde_yaml::to_string(&records)?;
        self.atomic_write(&self.call_records_path, &content)?;
        Ok(())
    }

    pub fn count_call_records_yaml(&self) -> Result<usize> {
        Ok(self.read_call_records_yaml().unwrap_or_default().len())
    }

    // Helpers
    fn soul_path(&self, name: &str) -> PathBuf {
        self.souls_dir.join(format!("{}.md", name))
    }

    fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn parse_soul_md(content: &str) -> Result<SoulProfile> {
        if let Some(rest) = content.strip_prefix("---\n") {
            if let Some(end) = rest.find("\n---") {
                let yaml_str = &rest[..end];
                let markdown = rest[end + 4..].trim().to_string();
                // Parse as generic Value first to handle both old and new formats
                let raw: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;

                let name = raw["name"].as_str().unwrap_or("unknown").to_string();
                let ismism_code = raw["ismism_code"].as_str().unwrap_or("0-0-0-0").to_string();

                // Domains: support both "domains" (list) and "domain" (list)
                let domains: Vec<String> = raw["domains"].as_sequence()
                    .or_else(|| raw["domain"].as_sequence())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let tags: Vec<String> = raw["tags"].as_sequence()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                // Trigger keywords from nested trigger object
                let trigger_keywords: Vec<String> = raw["trigger"].get("keywords")
                    .and_then(|v| v.as_sequence())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                // Extract field from description or use default
                let field = raw["field"].as_str()
                    .or_else(|| raw["description"].as_str().map(|d| d.split('|').next().unwrap_or("").trim()))
                    .unwrap_or("").to_string();

                // Default ontology/epistemology/teleology from ismism code or description
                let (ontology, epistemology, teleology) = if raw["ontology"].is_null() && raw["epistemology"].is_null() {
                    (String::new(), String::new(), String::new())
                } else {
                    (raw["ontology"].as_str().unwrap_or("").into(), raw["epistemology"].as_str().unwrap_or("").into(), raw["teleology"].as_str().unwrap_or("").into())
                };

                return Result::<SoulProfile>::Ok(SoulProfile {
                    name,
                    ismism_code,
                    field,
                    ontology,
                    epistemology,
                    teleology,
                    domains,
                    exclude_scenarios: vec![],
                    summon_count: raw["summon_count"].as_u64().unwrap_or(0) as u32,
                    effectiveness: EffectivenessStats::default(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    tags,
                    summon_prompt: markdown.trim_start_matches("|\n").to_string(),
                    practice_observations: vec![],
                    title: raw["title"].as_str().unwrap_or("").into(),
                    description: raw["description"].as_str().unwrap_or("").into(),
                    voice: raw["voice"].as_str().unwrap_or("").trim_start_matches("|\n").to_string(),
                    mind: raw["mind"].as_str().unwrap_or("").trim_start_matches("|\n").to_string(),
                    self_declare: raw["self_declare"].as_str().unwrap_or("").trim_start_matches("|\n").to_string(),
                    skills_expertise: raw["skills_expertise"].as_sequence().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
                    model: raw["model"].as_str().unwrap_or("").into(),
                    tools: raw["tools"].as_str().unwrap_or("").into(),
                    trigger_keywords,
                    compat: raw["compat"].as_sequence().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
                    incompat: raw["incompat"].as_sequence().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
                });
            }
        }
        Err(FoundationError::Validation("Invalid soul MD format: missing frontmatter".into()))
    }

    fn serialize_soul_md(profile: &SoulProfile) -> Result<String> {
        let frontmatter = SoulFrontmatter::from(profile.clone());
        let yaml = serde_yaml::to_string(&frontmatter)?;
        Ok(format!("---\n{}---\n\n{}", yaml, profile.summon_prompt))
    }

    pub fn count_soul_files(&self) -> Result<usize> {
        self.list_soul_names().map(|n| n.len())
    }
}

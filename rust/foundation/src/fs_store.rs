use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;

use crate::error::{FoundationError, Result};
use crate::models::*;

pub struct FileStore {
    souls_dir: PathBuf,
    souls_internal_dir: Option<PathBuf>,
    archive_dir: PathBuf,
    registry_path: PathBuf,
    call_records_path: PathBuf,
    registry_cache: RwLock<HashMap<String, RegistryEntry>>,
}

impl FileStore {
    pub fn new(souls_dir: PathBuf, archive_dir: PathBuf, registry_path: PathBuf, call_records_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&souls_dir)?;
        std::fs::create_dir_all(&archive_dir)?;
        // 清理上次崩溃残留的 .tmp 文件
        Self::cleanup_stale_tmp(&souls_dir);
        Self::cleanup_stale_tmp(&archive_dir);
        let _ = std::fs::remove_file(registry_path.with_extension("tmp"));
        let _ = std::fs::remove_file(call_records_path.with_extension("tmp"));
        let store = FileStore {
            souls_dir,
            souls_internal_dir: None,
            archive_dir,
            registry_path,
            call_records_path,
            registry_cache: RwLock::new(HashMap::new()),
        };
        store.reload_registry()?;
        Ok(store)
    }

    /// 设置内部魂目录（由部署者通过环境变量 WANMINFAN_SOULS_INTERNAL_DIR 指定）
    pub fn set_souls_internal_dir(&mut self, dir: PathBuf) {
        if dir.exists() {
            tracing::info!("Internal souls dir: {:?}", dir);
            self.souls_internal_dir = Some(dir);
        } else {
            tracing::warn!("Internal souls dir does not exist: {:?}", dir);
        }
    }

    // Soul operations with YAML frontmatter + Markdown body
    pub fn read_soul(&self, name: &str) -> Result<SoulProfile> {
        // 优先查 internal 目录
        if let Some(ref internal_dir) = self.souls_internal_dir {
            let internal_path = internal_dir.join(format!("{}.md", name));
            if internal_path.exists() {
                let content = std::fs::read_to_string(&internal_path)?;
                return Self::parse_soul_md(&content);
            }
        }
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
        let mut names = std::collections::HashSet::new();
        // 扫描公开魂目录
        let scan_dir = |dir: &Path, names: &mut std::collections::HashSet<String>| -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if name != "README" {
                            names.insert(name.to_string());
                        }
                    }
                }
            }
            Ok(())
        };
        if self.souls_dir.exists() {
            scan_dir(&self.souls_dir, &mut names)?;
        }
        if let Some(ref internal_dir) = self.souls_internal_dir {
            if internal_dir.exists() {
                scan_dir(internal_dir, &mut names)?;
            }
        }
        let mut result: Vec<String> = names.into_iter().collect();
        result.sort();
        Ok(result)
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
        let mut cache = self.registry_cache.write();
        *cache = registry.souls.clone();
        Ok(())
    }

    pub fn reload_registry(&self) -> Result<()> {
        let registry = self.read_registry_raw()?;
        let mut cache = self.registry_cache.write();
        *cache = registry.souls;
        Ok(())
    }

    pub fn get_registry_entry(&self, name: &str) -> Option<RegistryEntry> {
        self.registry_cache.read().get(name).cloned()
    }

    pub fn list_registry_entries(&self) -> Result<Vec<(String, RegistryEntry)>> {
        let cache = self.registry_cache.read();
        Ok(cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    pub fn registry_entry_count(&self) -> usize {
        self.registry_cache.read().len()
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
        let target = std::path::Path::new(path).canonicalize()?;
        let archive_root = self.archive_dir.canonicalize().unwrap_or_else(|_| self.archive_dir.clone());
        if !target.starts_with(&archive_root) {
            return Err(FoundationError::Validation(format!("path escapes archive dir: {}", path)));
        }
        Ok(std::fs::read_to_string(&target)?)
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
        let mut records = self.read_call_records_yaml().unwrap_or_else(|e| {
            tracing::warn!("Failed to read existing call records, starting fresh: {}", e);
            vec![]
        });
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

    fn cleanup_stale_tmp(dir: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    Self::cleanup_stale_tmp(&p);
                } else if p.extension().map_or(false, |e| e == "tmp") {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
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

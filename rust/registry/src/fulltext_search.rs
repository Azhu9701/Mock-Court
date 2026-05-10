use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use foundation::{SoulProfile, Result, FoundationError};

/// 简单的内存全文搜索引擎
pub struct FulltextSearchEngine {
    documents: Arc<RwLock<HashMap<String, SearchDocument>>>,
}

/// 可搜索的文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub doc_type: String,
    pub created_at: DateTime<Utc>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub content: String,
    pub doc_type: String,
    pub created_at: DateTime<Utc>,
}

impl FulltextSearchEngine {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn index_document(&self, doc: SearchDocument) -> Result<()> {
        self.documents
            .write()
            .map_err(|_| FoundationError::InvalidState("Lock error".into()))?
            .insert(doc.id.clone(), doc);
        Ok(())
    }
    
    pub fn index_soul(&self, soul: &SoulProfile) -> Result<()> {
        let full_content = format!(
            "{} {} {} {} {}",
            soul.name,
            soul.description,
            soul.summon_prompt,
            soul.domains.join(", "),
            soul.self_declare
        );
        
        let doc = SearchDocument {
            id: soul.name.clone(),
            title: soul.name.clone(),
            content: full_content,
            doc_type: "soul".to_string(),
            created_at: Utc::now(),
        };
        
        self.index_document(doc)
    }
    
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let docs = self.documents
            .read()
            .map_err(|_| FoundationError::InvalidState("Lock error".into()))?;
        
        let lower_query = query.to_lowercase();
        let mut results: Vec<_> = docs
            .values()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&lower_query) || 
                doc.content.to_lowercase().contains(&lower_query)
            })
            .cloned()
            .map(|doc| SearchResult {
                id: doc.id,
                title: doc.title,
                content: doc.content,
                doc_type: doc.doc_type,
                created_at: doc.created_at,
            })
            .take(limit)
            .collect();
        
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }
    
    pub fn delete_document(&self, id: &str) -> Result<()> {
        self.documents
            .write()
            .map_err(|_| FoundationError::InvalidState("Lock error".into()))?
            .remove(id);
        Ok(())
    }
    
    pub fn clear(&self) -> Result<()> {
        self.documents
            .write()
            .map_err(|_| FoundationError::InvalidState("Lock error".into()))?
            .clear();
        Ok(())
    }
}

impl Default for FulltextSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

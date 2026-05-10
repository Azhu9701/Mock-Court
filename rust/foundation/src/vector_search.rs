use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::info;
use num_traits::Float;

/// 向量嵌入类型
pub type Embedding = Vec<f32>;

/// 向量文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub embedding: Embedding,
    pub metadata: HashMap<String, String>,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 向量搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
    pub content: String,
}

/// 简单的向量索引（使用线性搜索，适合中小规模数据）
pub struct SimpleVectorIndex {
    documents: Arc<RwLock<HashMap<String, VectorDocument>>>,
    dimension: usize,
}

impl SimpleVectorIndex {
    /// 创建新的向量索引
    pub fn new(dimension: usize) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            dimension,
        }
    }
    
    /// 添加文档
    pub fn add_document(&self, doc: VectorDocument) -> Result<(), VectorSearchError> {
        if doc.embedding.len() != self.dimension {
            return Err(VectorSearchError::DimensionMismatch {
                expected: self.dimension,
                got: doc.embedding.len(),
            });
        }
        
        self.documents
            .write()
            .map_err(|_| VectorSearchError::LockError)?
            .insert(doc.id.clone(), doc);
            
        Ok(())
    }
    
    /// 批量添加文档
    pub fn add_documents(&self, docs: Vec<VectorDocument>) -> Result<(), VectorSearchError> {
        for doc in docs {
            self.add_document(doc)?;
        }
        Ok(())
    }
    
    /// 删除文档
    pub fn remove_document(&self, id: &str) -> Result<(), VectorSearchError> {
        self.documents
            .write()
            .map_err(|_| VectorSearchError::LockError)?
            .remove(id);
        Ok(())
    }
    
    /// 余弦相似度搜索
    pub fn search(&self, query_embedding: &Embedding, limit: usize) -> Result<Vec<VectorSearchResult>, VectorSearchError> {
        if query_embedding.len() != self.dimension {
            return Err(VectorSearchError::DimensionMismatch {
                expected: self.dimension,
                got: query_embedding.len(),
            });
        }
        
        let documents = self.documents.read().map_err(|_| VectorSearchError::LockError)?;
        
        let mut results: Vec<(f32, &VectorDocument)> = documents
            .values()
            .map(|doc| {
                let score = cosine_similarity(query_embedding, &doc.embedding);
                (score, doc)
            })
            .collect();
        
        // 按分数降序排序
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 取前 limit 个
        Ok(results
            .into_iter()
            .take(limit)
            .map(|(score, doc)| VectorSearchResult {
                id: doc.id.clone(),
                score,
                metadata: doc.metadata.clone(),
                content: doc.content.clone(),
            })
            .collect())
    }
    
    /// 获取文档数量
    pub fn len(&self) -> Result<usize, VectorSearchError> {
        Ok(self.documents.read().map_err(|_| VectorSearchError::LockError)?.len())
    }
    
    pub fn is_empty(&self) -> Result<bool, VectorSearchError> {
        Ok(self.len()? == 0)
    }
    
    /// 清空索引
    pub fn clear(&self) -> Result<(), VectorSearchError> {
        self.documents.write().map_err(|_| VectorSearchError::LockError)?.clear();
        Ok(())
    }
    
    /// 创建随机测试嵌入
    pub fn random_embedding(&self) -> Embedding {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..self.dimension)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect()
    }
}

/// 计算两个向量的余弦相似度
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product = dot(a, b);
    let norm_a = norm(a);
    let norm_b = norm(b);
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

/// 计算点积
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// 计算向量的范数
fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// 向量搜索错误
#[derive(Debug, thiserror::Error)]
pub enum VectorSearchError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("Lock error")]
    LockError,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);
        
        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_add_and_search() {
        let index = SimpleVectorIndex::new(3);
        
        let doc1 = VectorDocument {
            id: Uuid::new_v4().to_string(),
            embedding: vec![1.0, 0.0, 0.0],
            metadata: HashMap::new(),
            content: "Test 1".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        let doc2 = VectorDocument {
            id: Uuid::new_v4().to_string(),
            embedding: vec![0.0, 1.0, 0.0],
            metadata: HashMap::new(),
            content: "Test 2".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        index.add_document(doc1.clone()).unwrap();
        index.add_document(doc2.clone()).unwrap();
        
        let results = index.search(&vec![1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, doc1.id);
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }
}

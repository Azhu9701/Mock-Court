use std::time::Duration;

use async_trait::async_trait;
use possession::tools::ToolHandler;

use crate::bing;

pub struct WebSearchTool {
    searxng_url: String,
    search_engine: String,
}

impl WebSearchTool {
    pub fn new(searxng_url: String, search_engine: String) -> Self {
        WebSearchTool { searxng_url, search_engine }
    }
}

#[async_trait]
impl ToolHandler for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网获取最新信息。用于查询魂自身知识库中没有的实时信息、新闻、数据等。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词，使用中文短语更佳"
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| foundation::FoundationError::Validation(format!("参数解析失败: {e}")))?;

        let query = args["query"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 query 参数".into()))?;

        match self.search_engine.as_str() {
            "searxng" => search_searxng(query, &self.searxng_url).await,
            "bing" => search_bing(query).await,
            _ => search_bing(query).await,
        }
    }
}

async fn search_bing(query: &str) -> foundation::Result<String> {
    let results = bing::search(query, 5).await
        .map_err(|e| foundation::FoundationError::Validation(e))?;

    if results.is_empty() {
        return Ok(format!("未搜索到与「{}」相关的结果。", query));
    }

    let mut md = format!("## web_search 结果: {}\n\n", query);
    md.push_str(&format!("共 {} 条结果，展示前 {} 条：\n\n", results.len(), results.len().min(5)));

    for (i, r) in results.iter().enumerate() {
        md.push_str(&format!("{}. **{}**\n", i + 1, r.title));
        md.push_str(&format!("   URL: {}\n", r.url));
        if !r.snippet.is_empty() {
            let snip = if r.snippet.len() > 300 {
                let boundary = r.snippet
                    .char_indices()
                    .take_while(|(i, _)| *i < 300)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &r.snippet[..boundary])
            } else {
                r.snippet.clone()
            };
            md.push_str(&format!("   摘要: {}\n", snip));
        }
        md.push('\n');
    }

    Ok(md)
}

async fn search_searxng(query: &str, searxng_url: &str) -> foundation::Result<String> {
    let search_url = format!(
        "{}/search?format=json&q={}&language=zh&pageno=1",
        searxng_url.trim_end_matches('/'),
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>()
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&search_url)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| foundation::FoundationError::Io(std::io::Error::other(e.to_string())))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        foundation::FoundationError::Io(std::io::Error::other(format!("响应读取失败: {e}")))
    })?;

    if !status.is_success() {
        return Err(foundation::FoundationError::Validation(format!(
            "SearXNG 返回错误 ({})", status.as_u16()
        )));
    }

    let search_data: SearxngRaw = serde_json::from_str(&body).map_err(|e| {
        foundation::FoundationError::Validation(format!("SearXNG 响应解析失败: {e}"))
    })?;

    let results = search_data.results.unwrap_or_default();
    if results.is_empty() {
        return Ok(format!("未搜索到与「{}」相关的结果。", query));
    }

    let top = results.iter().take(5);
    let mut md = format!("## web_search 结果: {}\n\n", query);
    md.push_str(&format!("共 {} 条结果，展示前 {} 条：\n\n", results.len(), 5.min(results.len())));

    for (i, r) in top.enumerate() {
        md.push_str(&format!("{}. **{}**\n", i + 1, r.title));
        md.push_str(&format!("   URL: {}\n", r.url));
        if let Some(ref snippet) = r.content {
            let snip = if snippet.len() > 300 {
                let boundary = snippet
                    .char_indices()
                    .take_while(|(i, _)| *i < 300)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &snippet[..boundary])
            } else {
                snippet.clone()
            };
            md.push_str(&format!("   摘要: {}\n", snip));
        }
        md.push('\n');
    }

    Ok(md)
}

#[derive(Debug, serde::Deserialize)]
struct SearxngRaw {
    results: Option<Vec<SearxngItem>>,
}

#[derive(Debug, serde::Deserialize)]
struct SearxngItem {
    title: String,
    url: String,
    content: Option<String>,
}

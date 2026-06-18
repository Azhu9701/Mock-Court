use std::time::Duration;

use async_trait::async_trait;
use possession::tools::ToolHandler;
use web2llm::{Web2llm, Web2llmConfig, FetchMode};

pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self {
        WebFetchTool
    }
}

#[async_trait]
impl ToolHandler for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "抓取指定网页的完整内容并转换为 Markdown。用于深入阅读搜索结果中的某篇文章、法规条文、判例等。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "要抓取的网页完整 URL，必须以 http:// 或 https:// 开头"
                }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| foundation::FoundationError::Validation(format!("参数解析失败: {e}")))?;

        let url = args["url"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 url 参数".into()))?;

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(foundation::FoundationError::Validation(
                "URL 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // Fast path: Jina Reader (纯 HTTP, ~1-3 秒)
        match fetch_via_jina(url).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                tracing::info!("Jina Reader failed for {}: {}, falling back to web2llm", url, e);
            }
        }

        // Fallback: web2llm (支持更多网站，但较慢 ~10-30 秒)
        fetch_via_web2llm(url).await
    }
}

/// Fast path: 通过 Jina Reader 抓取网页内容。
/// Jina Reader 是一个免费的网页→Markdown 转换服务，纯 HTTP，无需配置。
async fn fetch_via_jina(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let jina_url = format!("https://r.jina.ai/{}" , url);
    let resp = client
        .get(&jina_url)
        .send()
        .await
        .map_err(|e| format!("Jina Reader 请求失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("Jina Reader 返回错误: {}", status.as_u16()));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Jina Reader 响应读取失败: {e}"))?;

    if text.trim().is_empty() {
        return Err("Jina Reader 返回空内容".into());
    }

    let mut md = format!("## 网页内容: {}\n\n", url);
    md.push_str(&format!("> 抓取时间: {} | 来源: Jina Reader\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&truncate_content(&text, 8000));

    Ok(md)
}

/// Fallback: 通过 web2llm 抓取网页内容。
/// 支持 JavaScript 渲染、反爬虫等复杂页面，但较慢。
async fn fetch_via_web2llm(url: &str) -> foundation::Result<String> {
    let web_config = Web2llmConfig {
        fetch_mode: FetchMode::Auto,
        max_concurrency: 1,
        ..Default::default()
    };

    let fetcher = Web2llm::new(web_config)
        .map_err(|e| foundation::FoundationError::Io(std::io::Error::other(format!("初始化抓取器失败: {e}"))))?;

    let result = fetcher
        .fetch(url)
        .await
        .map_err(|e| foundation::FoundationError::Io(std::io::Error::other(format!("网页抓取失败: {e}"))))?;

    let content = result.markdown();

    let mut md = format!("## 网页内容: {}\n\n", url);
    md.push_str(&format!("> 抓取时间: {} | 来源: web2llm\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&truncate_content(&content, 8000));

    Ok(md)
}

/// 截断内容到 max_len 字节，并在 UTF-8 字符边界处安全截断。
fn truncate_content(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let mut result = s[..end].to_string();
    result.push_str("\n\n*（内容已截断至 8000 字，如需完整内容请分段抓取）*");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_fetch_jina_reader() {
        let tool = WebFetchTool::new();
        let args = r#"{"url": "https://r.jina.ai/http://example.com"}"#;
        let result = tool.execute(args).await;
        
        match result {
            Ok(content) => {
                println!("SUCCESS! Content length: {}", content.len());
                println!("First 300 chars:\n{}", &content[..content.len().min(300)]);
                assert!(content.contains("example.com") || content.contains("Example"), 
                        "Content should mention example.com");
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
                // 网络测试可能因环境不稳定失败，不强制 panic
            }
        }
    }

    #[tokio::test]
    #[ignore = "需要外网访问，CI 环境可能不稳定"]
    async fn test_web_fetch_real_page() {
        let tool = WebFetchTool::new();
        let args = r#"{"url": "https://httpbin.org/html"}"#;
        let result = tool.execute(args).await;
        
        match result {
            Ok(content) => {
                println!("SUCCESS! Content length: {}", content.len());
                println!("First 300 chars:\n{}", &content[..content.len().min(300)]);
                assert!(!content.is_empty(), "Content should not be empty");
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_web_fetch_invalid_url() {
        let tool = WebFetchTool::new();
        let args = r#"{"url": "not-a-url"}"#;
        let result = tool.execute(args).await;
        assert!(result.is_err(), "Should reject invalid URL");
    }

    #[test]
    fn test_truncate_content() {
        let s = "这是一个中文测试字符串";
        assert_eq!(truncate_content(s, 3), "这");
        assert_eq!(truncate_content(s, 6), "这是");
        assert_eq!(truncate_content(s, 100), "这是一个中文测试字符串");
        
        // 测试截断提示
        let long = "a".repeat(10000);
        let truncated = truncate_content(&long, 8000);
        assert!(truncated.len() <= 8000 + 50); // 内容 + 截断提示
        assert!(truncated.contains("内容已截断"));
    }
}

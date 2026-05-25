use std::time::Duration;

use base64::Engine;

const BING_SEARCH_URL: &str = "https://www.bing.com/search";

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// 通过 Bing HTML 接口搜索
pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let params = [
        ("q", query),
        ("count", "20"),
    ];
    let resp = client
        .get(BING_SEARCH_URL)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("Bing 请求失败: {e}"))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Bing 响应读取失败: {e}"))?;

    if !status.is_success() {
        return Err(format!("Bing 返回错误 ({})", status.as_u16()));
    }

    let results = parse_html_results(&body, limit);
    if results.is_empty() {
        return Err("未搜索到结果".into());
    }

    Ok(results)
}

/// 轻量搜索：只取 title + snippet + URL，不抓网页
pub async fn search_quick(query: &str, limit: usize) -> Result<String, String> {
    let results = search(query, limit).await?;

    let mut md = String::new();
    md.push_str(&format!(
        "> 以下是通过 Bing 实时搜索「{}」获取的背景信息，供分析参考。\n",
        query
    ));
    md.push_str(&format!("> 共 {} 条来源\n\n", results.len()));

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

fn parse_html_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Bing 搜索结果在 <li class="b_algo"> 中
    let blocks: Vec<&str> = html.split("<li class=\"b_algo\"").skip(1).collect();

    for block in blocks {
        // 提取标题和链接
        let (title, url) = extract_title_and_url(block);
        // 提取摘要
        let snippet = extract_snippet(block);

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
            });
            if results.len() >= limit {
                break;
            }
        }
    }

    // fallback：如果标准解析失败，尝试宽松模式
    if results.is_empty() {
        results = parse_html_fallback(html, limit);
    }

    results
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// 从 Bing 跟踪链接中解析真实 URL。
/// Bing 的链接格式: https://www.bing.com/ck/a?...&u=a1<base64>&...
/// 其中 u= 参数的值以 a1 为前缀，后面是 base64 编码的真实 URL。
fn resolve_bing_url(bing_url: &str) -> String {
    // 如果不是 Bing 跟踪链接，直接返回
    if !bing_url.contains("/ck/a") {
        return bing_url.to_string();
    }

    // 先解码 HTML 实体（&amp; → &）
    let decoded = decode_html_entities(bing_url);

    // 提取 u= 参数
    if let Some(u_pos) = decoded.find("u=") {
        let after_u = &decoded[u_pos + 2..];
        // u 参数值可能在 & 处结束
        let u_value = after_u.split('&').next().unwrap_or(after_u);

        // 去掉 a1 前缀（如果存在）
        let b64_data = if u_value.starts_with("a1") {
            &u_value[2..]
        } else {
            u_value
        };

        // base64 解码（Bing 使用无填充的标准 base64）
        let engine = base64::engine::GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            base64::engine::general_purpose::NO_PAD,
        );
        if let Ok(bytes) = engine.decode(b64_data) {
            if let Ok(url) = String::from_utf8(bytes) {
                return percent_decode_url(&url);
            }
        }
    }

    // 解析失败，回退到原始 URL
    decoded
}

/// 对 URL 中的 percent-encoded 字符进行解码（如 %E4%BA%BA → 人）
fn percent_decode_url(input: &str) -> String {
    let mut decoded = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    decoded.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).to_string()
}

fn extract_title_and_url(block: &str) -> (String, String) {
    // 找 <h2> 标签内的 <a href="...">
    let h2_start = match block.find("<h2") {
        Some(i) => i,
        None => return (String::new(), String::new()),
    };
    let h2_end = match block[h2_start..].find("</h2>") {
        Some(i) => h2_start + i + 5,
        None => block.len(),
    };
    let h2_content = &block[h2_start..h2_end];

    // 提取 href 并解析真实 URL
    let raw_url = extract_attr(h2_content, "href=\"");
    let url = resolve_bing_url(&raw_url);
    // 提取标题文本（去掉标签）
    let title = strip_tags(h2_content);

    (title, url)
}

fn extract_snippet(block: &str) -> String {
    // 摘要通常在 <p> 标签中，在标题之后
    let h2_end = match block.find("</h2>") {
        Some(i) => i + 5,
        None => 0,
    };
    let after_h2 = &block[h2_end..];

    // 找第一个 <p> 标签
    if let Some(p_start) = after_h2.find("<p") {
        let p_content_start = match after_h2[p_start..].find(">") {
            Some(i) => p_start + i + 1,
            None => return String::new(),
        };
        let p_content = &after_h2[p_content_start..];
        let p_end = match p_content.find("</p>") {
            Some(i) => i,
            None => return String::new(),
        };
        strip_tags(&p_content[..p_end])
    } else {
        String::new()
    }
}

fn extract_attr(text: &str, attr_prefix: &str) -> String {
    if let Some(start) = text.find(attr_prefix) {
        let after = &text[start + attr_prefix.len()..];
        if let Some(end) = after.find('"') {
            return after[..end].to_string();
        }
    }
    String::new()
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

fn parse_html_fallback(html: &str, limit: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // 更宽松的匹配：找所有带 href 的 a 标签
    let mut pos = 0;
    while let Some(a_start) = html[pos..].find("<a href=\"") {
        let abs_start = pos + a_start;
        let after_href = &html[abs_start + 9..];
        let quote_end = match after_href.find('"') {
            Some(i) => i,
            None => break,
        };
        let url = &after_href[..quote_end];

        // 跳过 Bing 内部链接
        if url.starts_with("/") || url.contains("bing.com") || url.contains("microsoft.com") {
            pos = abs_start + 1;
            continue;
        }

        // 提取 a 标签内的文本作为标题
        let a_close = match after_href.find(">") {
            Some(i) => i,
            None => {
                pos = abs_start + 1;
                continue;
            }
        };
        let a_text_start = a_close + 1;
        let a_end_tag = match after_href[a_text_start..].find("</a>") {
            Some(i) => i,
            None => {
                pos = abs_start + 1;
                continue;
            }
        };
        let title = strip_tags(&after_href[a_text_start..a_text_start + a_end_tag]);

        let resolved_url = resolve_bing_url(url);
        if !title.is_empty() && resolved_url.starts_with("http") {
            results.push(SearchResult {
                title,
                url: resolved_url,
                snippet: String::new(),
            });
            if results.len() >= limit {
                break;
            }
        }

        pos = abs_start + 1;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_tags() {
        assert_eq!(strip_tags("<b>hello</b> world"), "hello world");
    }

    #[test]
    fn test_extract_attr() {
        assert_eq!(
            extract_attr(r#"href="https://example.com""#, "href=\""),
            "https://example.com"
        );
    }

    #[tokio::test]
    async fn test_bing_search() {
        let results = search("Rust 编程语言", 3).await;
        match results {
            Ok(r) => {
                println!("Bing 搜索成功，找到 {} 条结果:", r.len());
                for (i, item) in r.iter().enumerate() {
                    println!("  {}. {}", i + 1, item.title);
                    println!("     URL: {}", item.url);
                    println!("     摘要: {}", item.snippet);
                }
                assert!(!r.is_empty(), "搜索结果不应为空");
            }
            Err(e) => {
                println!("Bing 搜索失败: {}", e);
                // 网络测试可能因环境不稳定失败，不强制 panic
            }
        }
    }

    #[tokio::test]
    async fn test_bing_search_quick() {
        let md = search_quick("人工智能", 3).await;
        match md {
            Ok(text) => {
                println!("Bing quick search 结果:\n{}", text);
                assert!(text.contains("人工智能"), "结果应包含搜索关键词");
            }
            Err(e) => {
                println!("Bing quick search 失败: {}", e);
            }
        }
    }

    #[test]
    fn test_resolve_bing_url() {
        // 真实的 Bing 跟踪链接（人工智能 - 百度百科）
        let bing_url = "https://www.bing.com/ck/a?!&&p=e340eadb67c31074688cce4d3656a47df1cb428664a675f1d6c4c83e4b61e7a2JmltdHM9MTc3OTQ5NDQwMA&ptn=3&ver=2&hsh=4&fclid=322dad9c-a218-6124-24c2-bafea3276082&u=a1aHR0cHM6Ly9iYWlrZS5iYWlkdS5jb20vaXRlbS8lRTQlQkElQkElRTUlQjclQTUlRTYlOTklQkElRTglODMlQkQvOTE4MA&ntb=1";
        let resolved = resolve_bing_url(bing_url);
        assert_eq!(resolved, "https://baike.baidu.com/item/人工智能/9180");

        // HTML 实体编码的链接也应正确解析
        let bing_url_encoded = "https://www.bing.com/ck/a?!&amp;&amp;p=...&amp;u=a1aHR0cHM6Ly9leGFtcGxlLmNvbQ&amp;ntb=1";
        assert_eq!(resolve_bing_url(bing_url_encoded), "https://example.com");

        // 非 Bing 链接应原样返回
        let direct = "https://example.com/path";
        assert_eq!(resolve_bing_url(direct), "https://example.com/path");
    }
}

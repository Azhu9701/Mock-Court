use std::time::Duration;

const DUCKDUCKGO_HTML_URL: &str = "https://html.duckduckgo.com/html";

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// 通过 DuckDuckGo HTML 接口搜索
pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.0")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let params = [("q", query), ("kl", "zh-cn")];
    let resp = client
        .get(DUCKDUCKGO_HTML_URL)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("DuckDuckGo 请求失败: {e}"))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("DuckDuckGo 响应读取失败: {e}"))?;

    if !status.is_success() {
        return Err(format!("DuckDuckGo 返回错误 ({})", status.as_u16()));
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
        "> 以下是通过 DuckDuckGo 实时搜索「{}」获取的背景信息，供分析参考。\n",
        query
    ));
    md.push_str(&format!("> 共 {} 条来源\n\n",
        results.len()
    ));

    for (i, r) in results.iter().enumerate() {
        md.push_str(&format!("{}. **{}**\n", i + 1, r.title));
        md.push_str(&format!("   URL: {}\n", r.url));
        if !r.snippet.is_empty() {
            md.push_str(&format!("   摘要: {}\n", r.snippet));
        }
        md.push('\n');
    }

    Ok(md)
}

fn parse_html_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // DuckDuckGo HTML 结果格式：
    // <div class="result"> ... <a class="result__a" href="...">标题</a> ... <a class="result__snippet">摘要</a> ... </div>
    let result_blocks: Vec<&str> = html
        .split("<div class=\"results\"")
        .skip(1)
        .collect();

    for block in result_blocks {
        let title = extract_between(block, "<a class=\"result__a\"", "</a>");
        let url = extract_attr(&title, "href=");
        let title_text = strip_tags(&title);

        let snippet_block = extract_between(block, "<a class=\"result__snippet\"", "</a>");
        let snippet_text = strip_tags(&snippet_block);

        if !title_text.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title: title_text,
                url: decode_html_entities(&url),
                snippet: snippet_text,
            });
            if results.len() >= limit {
                break;
            }
        }
    }

    // fallback：如果上面的解析失败，尝试更宽松的模式
    if results.is_empty() {
        results = parse_html_fallback(html, limit);
    }

    results
}

fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = match text.find(start) {
        Some(i) => i + start.len(),
        None => return "",
    };
    let remaining = &text[start_idx..];
    let end_idx = match remaining.find(end) {
        Some(i) => i,
        None => return remaining,
    };
    &remaining[..end_idx]
}

fn extract_attr(text: &str, attr: &str) -> String {
    if let Some(start) = text.find(attr) {
        let after_attr = &text[start + attr.len()..];
        let quote = after_attr.chars().next().unwrap_or('"');
        let after_quote = &after_attr[1..];
        if let Some(end) = after_quote.find(quote) {
            return after_quote[..end].to_string();
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

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn parse_html_fallback(html: &str, limit: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let parts: Vec<&str> = html.split("<a rel=\"nofollow\" class=\"result__a\"").skip(1).collect();

    for part in parts {
        let url = extract_attr(part, "href=");
        let title_html = extract_between(part, ">", "</a>");
        let title = strip_tags(&title_html);

        let snippet_part = extract_between(part, "class=\"result__snippet\"", "</a>");
        let snippet_html = if snippet_part.contains('>') {
            let idx = snippet_part.rfind('>').unwrap_or(0);
            &snippet_part[idx + 1..]
        } else {
            snippet_part
        };
        let snippet = strip_tags(snippet_html);

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                url: decode_html_entities(&url),
                snippet,
            });
            if results.len() >= limit {
                break;
            }
        }
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
            extract_attr(r#"href="https://example.com""#, "href="),
            "https://example.com"
        );
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("a &amp; b"), "a & b");
    }
}

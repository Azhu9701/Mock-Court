use std::path::PathBuf;

use tarzi::search::{SearchEngine as TarziEngine, SearchEngineType};
use tarzi::config::Config as TarziConfig;
use web2llm::{Web2llm, Web2llmConfig, FetchMode};

/// 收魂流水线：搜索 → 抓取 → 保存 Markdown
pub struct SoulCollector {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CollectResult {
    pub name: String,
    pub raw_path: String,
    pub sources: usize,
    pub engine: String,
}

impl SoulCollector {
    pub fn new(data_dir: PathBuf) -> Self {
        SoulCollector { data_dir }
    }

    /// 搜索 + 逐条抓取 → 保存为 raw/{魂名}/搜索素材.md
    pub async fn collect(
        &self,
        name: &str,
        engine: Option<&str>,
        limit: usize,
    ) -> Result<CollectResult, String> {
        let engine_type = match engine.unwrap_or("baidu") {
            "baidu" => SearchEngineType::Baidu,
            "bing" => SearchEngineType::Bing,
            "google" => SearchEngineType::Google,
            "duckduckgo" => SearchEngineType::DuckDuckGo,
            e => return Err(format!("不支持的搜索引擎: {}", e)),
        };

        let engine_name = format!("{:?}", engine_type);

        // Use plain HTTP for engines that work without browser rendering
        let use_plain = matches!(engine_type, SearchEngineType::Bing | SearchEngineType::Google | SearchEngineType::DuckDuckGo);
        if !use_plain && engine_type == SearchEngineType::Baidu {
            return Err("百度搜索需要 chromedriver，当前环境暂不支持。请使用 bing/google/duckduckgo".into());
        }

        let mut config = TarziConfig::new();
        config.search.engine = format!("{:?}", engine_type).to_lowercase();
        if use_plain {
            config.fetcher.mode = "plain_request".to_string();
        }

        let mut search_engine = TarziEngine::from_config(&config);

        let search_results = search_engine
            .search(name, limit)
            .await
            .map_err(|e| format!("搜索失败: {}", e))?;

        if search_results.is_empty() {
            return Err(format!("未找到「{}」的相关结果", name));
        }

        // Fetch content for each result with web2llm
        let web_config = Web2llmConfig {
            fetch_mode: FetchMode::Auto,
            max_concurrency: 5,
            ..Default::default()
        };
        let fetcher = Web2llm::new(web_config)
            .map_err(|e| format!("初始化抓取器失败: {}", e))?;

        let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
        let fetched = fetcher.batch_fetch(urls).await;

        // Build Markdown output
        let mut md = String::new();
        md.push_str(&format!("# 收魂素材：{}\n\n", name));
        md.push_str(&format!("> 搜索引擎：{} | 抓取时间：{}\n\n", engine_name, chrono::Utc::now().format("%Y-%m-%d %H:%M")));
        md.push_str("---\n\n");

        let mut sources = 0usize;
        for (result, (_raw_url, fetch_res)) in search_results.into_iter().zip(fetched.into_iter()) {
            md.push_str(&format!("## {}\n\n", result.title));
            md.push_str(&format!("- **URL**: {}\n", result.url));
            md.push_str(&format!("- **摘要**: {}\n\n", result.snippet));

            match fetch_res {
                Ok(page_result) => {
                    let content = page_result.markdown();
                    if content.len() > 5000 {
                        md.push_str(&content[..5000]);
                        md.push_str("\n\n*（内容已截断至 5000 字）*\n\n");
                    } else {
                        md.push_str(&content);
                    }
                    sources += 1;
                }
                Err(e) => {
                    md.push_str(&format!("*抓取失败: {}*\n\n", e));
                }
            }
            md.push_str("\n---\n\n");
        }

        // Save to file
        let raw_dir = self.data_dir.join("raw").join(name);
        std::fs::create_dir_all(&raw_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
        let raw_path = raw_dir.join("搜索素材.md");
        std::fs::write(&raw_path, &md)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(CollectResult {
            name: name.to_string(),
            raw_path: raw_path.to_string_lossy().to_string(),
            sources,
            engine: engine_name,
        })
    }

    /// 议题搜索：搜索 + 抓取 → 返回 Markdown 字符串（不写文件）。
    /// 用于在附体前为魂提供议题的背景信息。
    pub async fn search_topic(
        &self,
        query: &str,
        engine: Option<&str>,
        limit: usize,
    ) -> Result<String, String> {
        let engine_type = match engine.unwrap_or("bing") {
            "baidu" => SearchEngineType::Baidu,
            "bing" => SearchEngineType::Bing,
            "google" => SearchEngineType::Google,
            "duckduckgo" => SearchEngineType::DuckDuckGo,
            e => return Err(format!("不支持的搜索引擎: {}", e)),
        };

        let engine_name = format!("{:?}", engine_type);
        let use_plain = matches!(engine_type, SearchEngineType::Bing | SearchEngineType::Google | SearchEngineType::DuckDuckGo);
        if !use_plain && engine_type == SearchEngineType::Baidu {
            return Err("百度搜索需要 chromedriver，当前环境暂不支持。请使用 bing/google/duckduckgo".into());
        }

        let mut config = TarziConfig::new();
        config.search.engine = format!("{:?}", engine_type).to_lowercase();
        if use_plain {
            config.fetcher.mode = "plain_request".to_string();
        }

        let mut search_engine = TarziEngine::from_config(&config);
        let search_results = search_engine.search(query, limit).await
            .map_err(|e| format!("搜索失败: {}", e))?;

        if search_results.is_empty() {
            return Ok(format!("（未搜索到与「{}」直接相关的结果，请魂依赖自身知识库判断）", query));
        }

        let web_config = Web2llmConfig {
            fetch_mode: FetchMode::Auto,
            max_concurrency: 5,
            ..Default::default()
        };
        let fetcher = Web2llm::new(web_config)
            .map_err(|e| format!("初始化抓取器失败: {}", e))?;

        let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();
        let fetched = fetcher.batch_fetch(urls).await;

        let mut md = String::new();
        md.push_str(&format!("> 以下是通过 {} 实时搜索「{}」获取的背景信息，供分析参考。\n", engine_name, query));
        md.push_str(&format!("> 共 {} 条来源\n\n", search_results.len()));

        for (result, (_raw_url, fetch_res)) in search_results.into_iter().zip(fetched.into_iter()) {
            md.push_str(&format!("### {}\n", result.title));
            md.push_str(&format!("- URL: {}\n", result.url));
            md.push_str(&format!("- 摘要: {}\n\n", result.snippet));

            match fetch_res {
                Ok(page_result) => {
                    let content = page_result.markdown();
                    if content.len() > 3000 {
                        md.push_str(&content[..3000]);
                        md.push_str("\n\n*（全文已截断至 3000 字）*\n\n");
                    } else {
                        md.push_str(&content);
                    }
                }
                Err(e) => {
                    md.push_str(&format!("*抓取失败: {}*\n\n", e));
                }
            }
            md.push_str("\n---\n\n");
        }

        Ok(md)
    }
}

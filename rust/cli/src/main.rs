use std::collections::HashMap;
use std::io::{self, Write};

use clap::{Parser, Subcommand};
use colored::*;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ── CLI Arguments ───────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "snake", about = "万民幡 · 多灵魂思辨系统 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 提交任务，灵魂分析并执行
    Run {
        /// 任务描述
        task: Vec<String>,
        /// 指定灵魂（逗号分隔），留空自动匹配
        #[arg(short, long)]
        souls: Option<String>,
        /// 附体模式: single, conference, debate, relay
        #[arg(short, long)]
        mode: Option<String>,
        /// 服务器地址
        #[arg(long, default_value = "http://localhost:3096")]
        server: String,
        /// API token
        #[arg(long)]
        token: Option<String>,
    },
    /// 列出所有可用灵魂
    Souls {
        #[arg(long, default_value = "http://localhost:3096")]
        server: String,
        #[arg(long)]
        token: Option<String>,
    },
    /// 查看历史会话
    Sessions {
        #[arg(long, default_value = "http://localhost:3096")]
        server: String,
        #[arg(long)]
        token: Option<String>,
    },
}

// ── API Types ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AnalyzeResponse {
    recommended_souls: Vec<RecommendedSoul>,
    recommended_mode: String,
    #[serde(default)]
    task_cards: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RecommendedSoul {
    name: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    score: f64,
}

#[derive(Debug, Serialize)]
struct StartRequest {
    task: String,
    souls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    judgment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    worry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unknown: Option<String>,
    #[serde(default)]
    search_topic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_cards: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct StartResponse {
    session_id: String,
    ws_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // 字段从 WS 事件反序列化，保留以便调试/未来使用
struct WsEvent {
    #[serde(alias = "type")]
    event_type: String,
    #[serde(default)]
    payload: String,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    soul_name: Option<String>,
    #[serde(default)]
    seq: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // 从 /souls 响应反序列化，字段保留以便调试
struct SoulEntry {
    name: String,
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

// ── HTTP Client ─────────────────────────────────────────────────────────

fn build_client(token: &Option<String>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Some(ref t) = token {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", t)).unwrap(),
        );
        builder = builder.default_headers(headers);
    }
    builder.build().unwrap()
}

async fn analyze_task(
    client: &reqwest::Client,
    server: &str,
    task: &str,
) -> Result<AnalyzeResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/possess/analyze", server);
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "task": task }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Analyze failed ({}): {}", status, body).into());
    }

    let text = resp.text().await?;

    // Try to parse as JSON first (non-streaming)
    if let Ok(json) = serde_json::from_str::<AnalyzeResponse>(&text) {
        return Ok(json);
    }

    // Parse SSE stream — the final event has "phase":"reviewed" or "phase":"complete"
    // with recommended_souls and recommended_mode
    let mut recommended_souls = Vec::new();
    let mut recommended_mode = String::new();
    let mut task_cards = HashMap::new();

    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                // Collect soul recommendations from matched events
                if let Some(souls) = json["souls"].as_array() {
                    recommended_souls = souls
                        .iter()
                        .map(|s| RecommendedSoul {
                            name: s["name"].as_str().unwrap_or("?").to_string(),
                            reason: s["rationale"].as_str().unwrap_or("").to_string(),
                            score: s["score"].as_f64().unwrap_or(0.0),
                        })
                        .collect();
                }
                // Collect mode recommendation
                if let Some(m) = json["recommended_mode"].as_str() {
                    recommended_mode = m.to_string();
                }
                if let Some(m) = json["mode"].as_str() {
                    recommended_mode = m.to_string();
                }
                // Collect task cards
                if let Some(cards) = json["task_cards"].as_object() {
                    for (k, v) in cards {
                        if let Some(s) = v.as_str() {
                            task_cards.insert(k.clone(), s.to_string());
                        }
                    }
                }
                // Check for final response
                if json.get("recommended_souls").is_some() {
                    if let Ok(ar) = serde_json::from_str::<AnalyzeResponse>(data) {
                        return Ok(ar);
                    }
                }
            }
        }
    }

    if !recommended_souls.is_empty() {
        Ok(AnalyzeResponse {
            recommended_souls,
            recommended_mode,
            task_cards,
        })
    } else {
        Err(format!("No soul recommendations found in response. Raw: {}", &text[..text.len().min(500)]).into())
    }
}

async fn start_possession(
    client: &reqwest::Client,
    server: &str,
    req: &StartRequest,
) -> Result<StartResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/possess", server);
    let resp = client.post(&url).json(req).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Start possession failed ({}): {}", status, body).into());
    }

    Ok(resp.json().await?)
}

async fn list_souls(
    client: &reqwest::Client,
    server: &str,
) -> Result<Vec<SoulEntry>, Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/souls", server);
    let resp = client.get(&url).send().await?;
    Ok(resp.json().await?)
}

// ── WebSocket Display ───────────────────────────────────────────────────

async fn connect_and_display(
    ws_url: &str,
    server: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert HTTP URL to WebSocket URL
    let ws_full = if server.starts_with("https") {
        format!("wss://{}{}", server.strip_prefix("https://").unwrap_or(server), ws_url)
    } else {
        format!("ws://{}{}", server.strip_prefix("http://").unwrap_or(server), ws_url)
    };

    eprintln!("  {} {}", "Connecting to".dimmed(), ws_full.dimmed());

    let (mut ws_stream, _) = connect_async(&ws_full).await?;
    eprintln!("  {} {}", "✓".green().bold(), "Connected. Waiting for soul responses...".dimmed());
    eprintln!();

    let mut soul_buffers: HashMap<String, String> = HashMap::new();
    let mut tool_call_count: u32 = 0;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        match msg {
            Message::Text(text) => {
                let event: WsEvent = match serde_json::from_str(&text) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("  {} WS parse error: {} — raw: {}", "⚠".yellow(), e, &text[..text.len().min(100)]);
                        continue;
                    }
                };

                match event.event_type.as_str() {
                    "session_started" => {
                        eprintln!("  {} Session started", "⚡".yellow());
                    }
                    "entry_classified" => {
                        eprintln!("  {} Mode: {}", "📋".blue(), event.payload.cyan());
                    }
                    "soul_started" => {
                        let name = event.soul_name.as_deref().unwrap_or("?");
                        eprintln!();
                        eprintln!("  {} {} is thinking...", "🧠".magenta(), name.bold().magenta());
                    }
                    "soul_token" => {
                        let name = event.soul_name.as_deref().unwrap_or("?");
                        let buf = soul_buffers.entry(name.to_string()).or_default();
                        buf.push_str(&event.payload);

                        // Print token inline
                        print!("{}", event.payload);
                        io::stdout().flush().ok();
                    }
                    "soul_done" => {
                        let name = event.soul_name.as_deref().unwrap_or("?");
                        eprintln!();
                        eprintln!("  {} {} finished", "✓".green().bold(), name.green());
                    }
                    "soul_error" => {
                        let name = event.soul_name.as_deref().unwrap_or("?");
                        eprintln!();
                        eprintln!("  {} {} error: {}", "✗".red().bold(), name.red(), event.payload.red());
                    }
                    "tool_call_started" => {
                        tool_call_count += 1;
                        let payload: serde_json::Value = serde_json::from_str(&event.payload).unwrap_or_default();
                        let tool_name = payload["tool_name"].as_str().unwrap_or("?");
                        let soul = payload["soul_name"].as_str().unwrap_or("?");
                        let args = payload["arguments"].as_str().unwrap_or("{}");

                        eprintln!();
                        eprintln!("  {} {} calling {} {}",
                            "🔧".yellow(),
                            soul.bold().yellow(),
                            tool_name.bold().cyan(),
                            truncate_args(args).dimmed()
                        );
                    }
                    "tool_result" => {
                        let payload: serde_json::Value = serde_json::from_str(&event.payload).unwrap_or_default();
                        let tool_name = payload["tool_name"].as_str().unwrap_or("?");
                        let result = payload["result"].as_str().unwrap_or("");

                        let preview = if result.len() > 300 {
                            format!("{}...", &result[..300])
                        } else {
                            result.to_string()
                        };
                        eprintln!("  {} {} → {}",
                            "└".dimmed(),
                            tool_name.dimmed(),
                            preview.dimmed()
                        );
                    }
                    "synthesis_started" => {
                        eprintln!();
                        eprintln!("  {} Dialectical synthesis in progress...", "⚗️".blue());
                    }
                    "synthesis_chunk" => {
                        print!("{}", event.payload.blue());
                        io::stdout().flush().ok();
                    }
                    "synthesis_done" => {
                        eprintln!();
                        eprintln!("  {} Synthesis complete", "✓".green().bold());
                    }
                    "collision" => {
                        eprintln!("  {} Semantic collision detected: {}",
                            "💥".red(),
                            event.payload.yellow()
                        );
                    }
                    "system" => {
                        eprintln!("  {} {}", "ℹ".blue(), event.payload.dimmed());
                    }
                    "cost" => {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.payload) {
                            let tokens = json["total_tokens"].as_u64().unwrap_or(0);
                            let cost = json["cost_usd"].as_f64().unwrap_or(0.0);
                            eprintln!("  {} Tokens: {} | Cost: ${:.4}",
                                "💰".yellow(),
                                tokens,
                                cost
                            );
                        }
                    }
                    "done" => {
                        eprintln!();
                        eprintln!("  {} {}", "🎉".green(), "Session complete!".green().bold());
                        if tool_call_count > 0 {
                            eprintln!("  {} Tool calls made: {}", "📊".blue(), tool_call_count);
                        }
                        break;
                    }
                    _ => {
                        // Debug: show unknown events
                        eprintln!("  [{}] {}", event.event_type.dimmed(), event.payload.chars().take(100).collect::<String>().dimmed());
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

fn truncate_args(args: &str) -> String {
    if args.len() <= 80 {
        args.to_string()
    } else {
        format!("{}...", &args[..80])
    }
}

// ── Commands ────────────────────────────────────────────────────────────

async fn cmd_run(
    task_words: Vec<String>,
    souls: Option<String>,
    mode: Option<String>,
    server: String,
    token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let task = task_words.join(" ");
    if task.is_empty() {
        eprintln!("{} Task description is required", "Error:".red().bold());
        std::process::exit(1);
    }

    let client = build_client(&token);

    // Step 1: Analyze
    eprintln!("  {} Analyzing task...", "🔍".blue());
    let analysis = analyze_task(&client, &server, &task).await?;

    eprintln!("  {} Recommended mode: {}", "📋".blue(), analysis.recommended_mode.cyan());
    eprintln!("  {} Recommended souls:", "👥".blue());
    for s in &analysis.recommended_souls {
        eprintln!("    {} {} (score: {:.2}) - {}",
            "•".dimmed(),
            s.name.bold(),
            s.score,
            s.reason.dimmed()
        );
    }

    // Determine souls to use
    let selected_souls: Vec<String> = if let Some(s) = souls {
        s.split(',').map(|x| x.trim().to_string()).collect()
    } else {
        analysis.recommended_souls.iter().map(|s| s.name.clone()).collect()
    };

    eprintln!();
    eprintln!("  {} Summoning: {}", "⚔️".yellow(),
        selected_souls.iter().map(|s| s.bold().to_string()).collect::<Vec<_>>().join(", ")
    );

    // Step 2: Start possession
    let start_req = StartRequest {
        task: task.clone(),
        souls: selected_souls,
        mode: mode.or(Some(analysis.recommended_mode)),
        topic: None,
        judgment: None,
        worry: None,
        unknown: None,
        search_topic: false,
        task_cards: if analysis.task_cards.is_empty() {
            None
        } else {
            Some(analysis.task_cards)
        },
    };

    let start_resp = start_possession(&client, &server, &start_req).await?;
    eprintln!("  {} Session: {}", "📌".blue(), start_resp.session_id.dimmed());

    // Step 3: Connect WebSocket and display
    connect_and_display(&start_resp.ws_url, &server).await?;

    Ok(())
}

async fn cmd_souls(
    server: String,
    token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client(&token);
    let souls = list_souls(&client, &server).await?;

    eprintln!("  {} {} souls available:", "👥".blue(), souls.len());
    eprintln!();
    for s in &souls {
        let domains = if s.domains.is_empty() {
            String::new()
        } else {
            format!(" [{}]", s.domains.join(", "))
        };
        eprintln!("    {}{}",
            s.name.bold(),
            domains.dimmed()
        );
    }
    Ok(())
}

// ── Main ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { task, souls, mode, server, token } => {
            cmd_run(task, souls, mode, server, token).await?;
        }
        Commands::Souls { server, token } => {
            cmd_souls(server, token).await?;
        }
        Commands::Sessions { server, token } => {
            let client = build_client(&token);
            let resp = client.get(format!("{}/api/v1/sessions", server)).send().await?;
            let sessions: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&sessions)?);
        }
    }

    Ok(())
}

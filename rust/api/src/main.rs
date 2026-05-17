mod collector;
mod error;
mod middleware;
mod ocr;
mod rate_limiter;
mod routes;
mod state;
mod store;
mod web_search_tool;
mod ws;

use std::sync::Arc;

use archive::ArchiveSystem;
use possession::PossessionEngine;
use registry::SoulRegistry;

use crate::collector::SoulCollector;
use crate::rate_limiter::RateLimiter;
use crate::state::AppState;
use crate::store::AppStore;
use crate::web_search_tool::WebSearchTool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    // 检测是否运行在主流云平台：请求 ipinfo.io 提取 org 字段比对已知云厂商
    // 失败或超时时静默跳过，不阻塞启动
    let _ = async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build();
        let client = match client {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Ok(resp) = client.get("https://ipinfo.io/json").send().await {
            if let Ok(body) = resp.text().await {
                let lower = body.to_lowercase();
                let is_cloud = lower.contains("\"org\":")
                    && (lower.contains("amazon") || lower.contains("aws")
                        || lower.contains("google cloud") || lower.contains("gcp")
                        || lower.contains("microsoft") || lower.contains("azure")
                        || lower.contains("alibaba") || lower.contains("aliyun")
                        || lower.contains("tencent cloud") || lower.contains("huawei cloud")
                        || lower.contains("oracle cloud") || lower.contains("digitalocean")
                        || lower.contains("cloudflare"));
                if is_cloud {
                    eprintln!("  [!] 检测到云平台部署环境。");
                    eprintln!("  本软件被设计用于服务被剥夺者的集体行动。");
                    eprintln!("  你确定你的使用场景符合这个目的吗？");
                    eprintln!();
                    tracing::warn!("Cloud platform deployment detected — moral reminder printed");
                }
            }
        }
    }.await;

    tracing::info!("Loading configuration...");
    let config = foundation::Config::load()?;

    let rate_limiter = load_rate_limiter();

    // 启动限流器过期 bucket 清理定时任务
    {
        let rl = rate_limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                rl.cleanup();
            }
        });
    }

    tracing::info!("Initializing store...");
    let data_dir = &config.data_dir;
    let store = Arc::new(AppStore::new(data_dir.to_str().unwrap())?);

    tracing::info!("Loading soul registry...");
    let registry = Arc::new(SoulRegistry::new(store.clone()).await?);

    tracing::info!("Initializing AI gateway...");
    let gateway = {
        let gateway = ai_gateway::GatewayRegistry::new();
        tracing::info!("Initializing LLM cache...");
        let cache = Arc::new(ai_gateway::cache::LlMCache::new(store.db(), 3600));
        gateway.set_cache(cache);
        Arc::new(gateway)
    };

    tracing::info!("Initializing archive system...");
    let archive = Arc::new(ArchiveSystem::new(store.clone()));

    tracing::info!("Initializing possession engine...");
    let mut engine = PossessionEngine::new(
        store.clone(),
        registry.clone(),
        gateway,
    );

    tracing::info!("Registering built-in tools...");
    engine.tool_registry_mut().register(std::sync::Arc::new(WebSearchTool::new(config.searxng_url.clone())));
    let engine = Arc::new(engine);

    let collector = Arc::new(SoulCollector::new(data_dir.to_path_buf(), config.searxng_url.clone()));

    let state = Arc::new(AppState {
        registry,
        engine: engine.clone(),
        archive,
        collector,
        config: Arc::new(config),
        auto_create_tasks: Arc::new(dashmap::DashMap::new()),
        interrogation_gates: Arc::new(dashmap::DashMap::new()),
    });

    let app = build_router(state, rate_limiter);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3096").await?;
    tracing::info!("API server listening on http://0.0.0.0:3096");

    let engine_for_shutdown = engine.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received");
        engine_for_shutdown.set_shutdown();
        tx.send(()).ok();
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await?;

    Ok(())
}

fn load_rate_limiter() -> Arc<RateLimiter> {
    let settings = config::Config::builder()
        .add_source(config::File::from(std::path::Path::new("config/default.yaml")))
        .add_source(config::File::from(std::path::Path::new("config/local.yaml")).required(false))
        .build()
        .unwrap_or_else(|_| config::Config::builder().build().unwrap());

    let enabled = settings.get_bool("rate_limit.enabled").unwrap_or(true);
    if !enabled {
        tracing::info!("Rate limiter disabled");
        return Arc::new(RateLimiter::new(f64::MAX, f64::MAX));
    }

    let rps = settings.get_float("rate_limit.requests_per_second").unwrap_or(30.0);
    let burst = settings.get_float("rate_limit.burst_size").unwrap_or(60.0);
    tracing::info!("Rate limiter enabled: {:.0} req/s, burst {:.0}", rps, burst);
    Arc::new(RateLimiter::new(rps, burst))
}

fn build_router(state: Arc<AppState>, rate_limiter: Arc<RateLimiter>) -> axum::Router {
    let api_router = routes::api_router();

    let app = axum::Router::new()
        .nest("/api/v1", api_router)
        .route(
            "/ws/possess/:session_id/:channel",
            axum::routing::get(ws::ws_handler),
        )
        .route(
            "/ws/souls/auto-create/:task_id",
            axum::routing::get(ws::auto_create_ws_handler),
        )
        .with_state(state);

    crate::middleware::apply_middleware(app, rate_limiter)
}

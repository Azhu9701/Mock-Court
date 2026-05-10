mod collector;
mod error;
mod middleware;
mod ocr;
mod routes;
mod state;
mod store;
mod ws;

use std::sync::Arc;

use archive::ArchiveSystem;
use possession::PossessionEngine;
use registry::SoulRegistry;

use crate::collector::SoulCollector;
use crate::state::AppState;
use crate::store::AppStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    tracing::info!("Initializing store...");
    let current_dir = std::env::current_dir()?;
    let data_dir = current_dir.join("data");
    let store = Arc::new(AppStore::new(data_dir.to_str().unwrap())?);

    tracing::info!("Loading soul registry...");
    let registry = Arc::new(SoulRegistry::new(store.clone()).await?);

    tracing::info!("Initializing AI gateway...");
    let gateway = Arc::new(ai_gateway::GatewayRegistry::new());

    tracing::info!("Initializing archive system...");
    let archive = Arc::new(ArchiveSystem::new(store.clone()));

    tracing::info!("Initializing possession engine...");
    let engine = Arc::new(PossessionEngine::new(
        store.clone(),
        registry.clone(),
        gateway,
    ));

    let collector = Arc::new(SoulCollector::new(data_dir));

    let state = Arc::new(AppState {
        registry,
        engine: engine.clone(),
        archive,
        collector,
    });

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3096").await?;
    tracing::info!("API server listening on http://127.0.0.1:3096");

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

fn build_router(state: Arc<AppState>) -> axum::Router {
    let api_router = routes::api_router();

    let app = axum::Router::new()
        .nest("/api/v1", api_router)
        .route(
            "/ws/possess/:session_id/:channel",
            axum::routing::get(ws::ws_handler),
        )
        .with_state(state);

    crate::middleware::apply_middleware(app)
}

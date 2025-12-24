use axum::{
    http::{header::CONTENT_TYPE, Method},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod error;
mod handlers;
mod jwt;
mod middleware;
mod models;
mod repository;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Only load .env file in development environment
    if is_development() {
        dotenvy::dotenv().ok();
        tracing::debug!("Loaded .env file for development");
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "limitless_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let pool = database::init_database().await?;
    tracing::info!("🗄️  Database connected and migrations applied");

    // Create repository as shared state
    let repo = Arc::new(repository::UserRepository::new(pool));

    // Setup CORS for frontend
    let frontend_origin =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let cors = CorsLayer::new()
        .allow_origin(frontend_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Build our application with routes
    let app = Router::new()
        .nest("/api", routes::create_routes(repo))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Bind to 0.0.0.0 in production, localhost in development
    let addr = if is_development() {
        SocketAddr::from(([127, 0, 0, 1], 8080))
    } else {
        SocketAddr::from(([0, 0, 0, 0], 8080))
    };
    tracing::info!("🎮 Limitless backend server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Setup graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down gracefully...");
        },
    }
}

/// Check if we're running in a development environment
/// This checks for common development indicators
fn is_development() -> bool {
    // Check if we're in debug mode (cargo build vs cargo build --release)
    if cfg!(debug_assertions) {
        return true;
    }

    // Check for explicit ENVIRONMENT variable
    if let Ok(env) = std::env::var("ENVIRONMENT") {
        return env.to_lowercase() == "development" || env.to_lowercase() == "dev";
    }

    // Check for Rust environment variable
    if let Ok(rust_env) = std::env::var("RUST_ENV") {
        return rust_env.to_lowercase() == "development" || rust_env.to_lowercase() == "dev";
    }

    // Check if .env file exists (common in development)
    std::path::Path::new(".env").exists()
}

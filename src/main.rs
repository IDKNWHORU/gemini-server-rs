use axum::{
    routing::get,
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HealthCheckResponse {
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Health check endpoint
    async fn health_check() -> impl IntoResponse {
        let response = HealthCheckResponse {
            status: "OK".to_string(),
        };
        (StatusCode::OK, Json(response))
    }

    // Data generation endpoint
    async fn generate() -> impl IntoResponse {
        "Hello, world!"
    }

    // Create the router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/generate", get(generate));

    // Bind to the address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}

use axum::response::{IntoResponse, Response}; // Import IntoResponse and Response here
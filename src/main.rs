mod discord_webhook;
mod gemini_client;
mod output;
mod prompt;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use discord_webhook::DiscordWebhookClient;
use dotenv::dotenv;
use gemini_client::{GeminiClient, GeminiResponse};
use output::clean_error_output;
use prompt::get_prompt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HealthCheckResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct RequestData {
    #[serde(rename = "errorOutput")]
    error_output: String,
    code: String,
    language: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Clone)]
struct AppState {
    discord_webhook_client: Option<Arc<DiscordWebhookClient>>,
    gemini_client: GeminiClient,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let web_hook_url = env::var("WEB_HOOK_URL").ok();

    let client = Client::new();
    let discord_webhook_client =
        web_hook_url.map(|url| Arc::new(DiscordWebhookClient::new(client.clone(), Some(url))));
    let gemini_client = GeminiClient::new(client.clone(), api_key.clone());

    let app_state = AppState {
        discord_webhook_client,
        gemini_client,
    };

    // Health check endpoint
    async fn health_check() -> impl IntoResponse {
        let response = HealthCheckResponse {
            status: "OK".to_string(),
        };
        (StatusCode::OK, Json(response))
    }

    // Data generation endpoint
    async fn generate(
        State(state): State<AppState>,
        Json(request_data): Json<RequestData>,
    ) -> impl IntoResponse {
        let error_output = clean_error_output(&request_data.error_output);
        let prompt = get_prompt(&request_data.language, &error_output, &request_data.code);

        match &state.gemini_client.count_tokens(&prompt).await {
            Ok(total_tokens) => {
                println!("Total tokens: {}", total_tokens);

                if let Some(discord_client) = &state.discord_webhook_client {
                    if let Err(e) = discord_client
                        .send_token_info(&request_data.language, *total_tokens, &error_output)
                        .await
                    {
                        eprintln!("Error sending Discord webhook: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error counting tokens: {}", e);
            }
        }

        let gemini_response_result: Result<GeminiResponse, reqwest::Error> =
            state.gemini_client.generate_content(&prompt).await;

        match gemini_response_result {
            Ok(gemini_response) => Ok(Json(gemini_response)),
            Err(e) => {
                if let Some(discord_client) = &state.discord_webhook_client {
                    discord_client
                        .send_error_log(&format!("Result error: {}", e))
                        .await
                        .ok();
                }

                let error_response = ErrorResponse {
                    message: format!("Request error: {}", e),
                };

                eprintln!("error {}", error_response.message);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response())
            }
        }
    }

    // Create the router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/generate", post(generate))
        .with_state(app_state);

    // Bind to the address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}

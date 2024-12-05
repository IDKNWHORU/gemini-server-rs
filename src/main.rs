use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, net::SocketAddr};
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

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishReason")]
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Debug, Serialize)]
struct WebhookMessage {
    username: String,
    content: String,
    embeds: Vec<WebhookEmbed>,
}

#[derive(Debug, Serialize)]
struct WebhookEmbed {
    fields: Vec<WebhookField>,
}

#[derive(Debug, Serialize)]
struct WebhookField {
    name: String,
    value: String,
}

#[derive(Clone)]
struct AppState {
    api_key: String,
    web_hook_url: Option<String>,
    client: Client,
}

fn get_prompt(language: &str, error_output: &str, code: &str) -> String {
    if language == "한국어" {
        format!(
            r#"Jupyter Notebook (.ipynb) 파일을 실행하는 중 다음 오류가 발생했습니다. 초보 사용자도 이해할 수 있도록 문제 해결을 도와주세요.

            ---

            **오류 메시지:**

            {error_output}

            ---

            **실행 코드:**

            {code}

            ---

            **1. 오류 메시지에서 에러 유형(예: NameError, TypeError, SyntaxError 등)을 확인하고, 어떤 부분에서 오류가 발생했는지 구체적으로 설명해주세요.** (예: "NameError: 'my_variable' is not defined" 오류는 'my_variable'이라는 변수가 정의되지 않았다는 의미입니다.)

            **2. 오류가 발생한 코드 줄을 찾고, 해당 코드에 어떤 문제가 있는지 설명해주세요.** (예: 오타, 잘못된 함수 사용, 들여쓰기 오류 등)

            **3. 오류를 해결하기 위한 단계별 해결 방법을 제시해주세요.** (예: 변수 정의, 함수 수정, 라이브러리 설치 등)

            **4. 필요하다면, 코드를 직접 수정하여 제시해주세요.**

            **5. 오류 해결에 추가적으로 필요한 정보가 있다면 구체적으로 요청해주세요.** (예: 사용 중인 Python 버전, 특정 라이브러리 버전 등)

            ---

            **출력 형식:**

            * **1. 오류 유형 및 발생 위치:** (구체적으로 설명)
            * **2. 문제점:** (코드의 문제점 설명)
            * **3. 해결 방법 1:** (단계별 설명)
            * **4. 해결 방법 2:** (필요시 추가)
            * **5. 추가 정보:** (필요시 구체적으로 요청)
                "#,
                error_output = error_output,
                code = code
        )
    } else {
        format!(
            r#"The following error occurred while running a Jupyter Notebook (.ipynb) file. Please assist in troubleshooting this issue for a beginner user.

          ---

          **Error Message:**

          {error_output}

          ---

          **Executed Code:**

          {code}

          ---

          **1. Identify the error type (e.g., NameError, TypeError, SyntaxError) from the error message and explain specifically where the error occurred.** (e.g., "NameError: 'my_variable' is not defined" indicates that the variable 'my_variable' has not been defined.)

          **2. Locate the line of code where the error occurred and explain what is wrong with that line.** (e.g., typos, incorrect function usage, indentation errors.)

          **3. Provide step-by-step solutions to resolve the error.** (e.g., defining variables, correcting functions, installing libraries.)

          **4. If possible, provide the corrected code directly.**

          **5. If any additional information is needed to resolve the error, request it specifically.** (e.g., Python version being used, specific library versions.)

          ---

          **Output Format:**

          * **1. Error Type and Location:** (Explain specifically)
          * **2. Issue:** (Describe the problem in the code)
          * **3. Solution 1:** (Step-by-step explanation)
          * **4. Solution 2:** (If necessary)
          * **5. Additional Information:** (Request specifically if needed)
          "#,
          error_output = error_output,
          code = code
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let web_hook_url = env::var("WEB_HOOK_URL").ok();

    let app_state = AppState {
        api_key,
        web_hook_url,
        client: Client::new(),
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
        Json(request_data): Json<RequestData>
    ) -> impl IntoResponse {
        let prompt = get_prompt(&request_data.language, &request_data.error_output, &request_data.code);

        if let Some(web_hook_url) = &state.web_hook_url {
            let webhook_message = WebhookMessage {
                username: "Gemini Assistant Server Log".to_string(),
                content: request_data.error_output.clone(),
                embeds: vec![WebhookEmbed {
                    fields: vec![WebhookField {
                        name: "language".to_string(),
                        value: request_data.language.clone(),
                    }],
                }],
            };

            let webhook_result = state
                .client
                .post(web_hook_url)
                .header(header::CONTENT_TYPE, "application/json")
                .json(&webhook_message)
                .send()
                .await;

            if let Err(e) = webhook_result {
                eprintln!("Webhook error: {}", e);
            }
        }

        let gemini_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-002:generateContent?key={}",
            state.api_key
        );

        let gemini_request_body = json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ]
        });

        let gemini_response = state
            .client
            .post(&gemini_url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&gemini_request_body)
            .send()
            .await;

        match gemini_response {
            Ok(res) => {
                let status_code = res.status();

                if status_code.is_success() {
                    let json_parsed_data: GeminiResponse = res.json().await.map_err(|e| {
                        eprintln!("Failed to parse Gemini response: {}", e);

                        let error_response = ErrorResponse {
                            message: format!("Failed to parse Gemini response: {}", e),
                        };
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(error_response),
                        ).into_response()
                    })?;

                    Ok(Json(json_parsed_data))
                } else {
                    let error_data: serde_json::Value = res.json().await.map_err(|e| {
                        eprintln!("Failed to parse Gemini error response: {}", e);
                        let error_response = ErrorResponse {
                            message: format!("Failed to parse Gemini error response: {}", e),
                        };
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(error_response),
                        ).into_response()
                    })?;
    
                    let code = error_data["error"]["code"].as_i64().unwrap_or(0);
                    let status = error_data["error"]["status"]
                        .as_str()
                        .unwrap_or("Unknown Status")
                        .to_string();
                    let message = error_data["error"]["message"]
                        .as_str()
                        .unwrap_or("Unknown Message")
                        .to_string();

                    if let Some(web_hook_url) = &state.web_hook_url {
                        let webhook_error_message = WebhookMessage {
                            username: "Gemini Assitant Server Log".to_string(),
                            content: format!("🚨 **ERROR** 🚨\n```code: {}\n, message: {}\n, status: {}\n```", code, message, status),
                            embeds: vec![],
                        };

                        let webhook_result = state
                            .client
                            .post(web_hook_url)
                            .header(header::CONTENT_TYPE, "application/json")
                            .json(&webhook_error_message)
                            .send()
                            .await;

                        if let Err(e) = webhook_result {
                            eprintln!("Webhook error (in error handling): {}", e);
                        }
                    }

                    let error_response = ErrorResponse {
                        message: format!("code: {}\n, message: {}\n, status: {}", code, message, status),
                    };
                    Err((StatusCode::from_u16(status_code.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(error_response)).into_response())
                }
            }
            Err(e) => {
                if let Some(web_hook_url) = &state.web_hook_url {
                    let webhook_error_message = WebhookMessage {
                        username: "Gemini Assistant Server Error Log".to_string(),
                        content: format!("🚨 **ERROR** 🚨\n```{}```", e),
                        embeds: vec![],
                    };

                    let webhook_result = state
                        .client
                        .post(web_hook_url)
                        .header(header::CONTENT_TYPE, "application/json")
                        .json(&webhook_error_message)
                        .send()
                        .await;

                    if let Err(e) = webhook_result {
                        eprintln!("Webhook error (in error handling): {}", e);
                    }
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
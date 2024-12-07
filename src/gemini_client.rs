use reqwest::{header, Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct GeminiRequestBody {
    contents: Vec<GeminiRequestContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiRequestContent {
    parts: Vec<GeminiRequestPart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiRequestPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    candidates: Vec<GeminiResponseCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponseCandidate {
    content: GeminiResponseContent,
    #[serde(rename = "finishReason")]
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponsePart {
    text: String,
}

#[derive(Debug, Serialize)]
struct CountTokenRequestBody {
    contents: Vec<CountTokenContent>,
}

#[derive(Debug, Serialize)]
struct CountTokenContent {
    parts: Vec<CountTokenPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountTokenPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountTokensResponse {
    #[serde(rename = "totalTokens")]
    total_tokens: i32,
}

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(client: Client, api_key: String) -> Self {
        Self { client, api_key }
    }

    pub async fn count_tokens(&self, prompt: &str) -> Result<i32, ReqwestError> {
        let url = format!(
      "https://generativelanguage.googleapis.com/v1beta/models/gemini-exp-1121:countTokens?key={}",
      self.api_key
    );

        let request_body = CountTokenRequestBody {
            contents: vec![CountTokenContent {
                parts: vec![CountTokenPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let res = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_body: CountTokensResponse = res.json().await?;

        Ok(response_body.total_tokens)
    }

    pub async fn generate_content(&self, prompt: &str) -> Result<GeminiResponse, ReqwestError> {
        let gemini_url = format!(
      "https://generativelanguage.googleapis.com/v1beta/models/gemini-exp-1121:generateContent?key={}",
      self.api_key
    );

        let gemini_request_body = GeminiRequestBody {
            contents: vec![GeminiRequestContent {
                parts: vec![GeminiRequestPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let res = self
            .client
            .post(&gemini_url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&gemini_request_body)
            .send()
            .await?;

        res.json().await
    }
}

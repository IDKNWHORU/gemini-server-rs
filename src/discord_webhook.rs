use reqwest::{header, Client, Error as ReqwestError};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct DiscordWebhookMessage {
    username: String,
    content: String,
    embeds: Vec<DiscordWebhookEmbed>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DiscordWebhookEmbed {
    fields: Vec<DiscordWebhookField>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DiscordWebhookField {
    name: String,
    value: String,
}

#[derive(Clone)]
pub struct DiscordWebhookClient {
    client: Client,
    web_hook_url: Option<String>,
}

impl DiscordWebhookClient {
    pub fn new(client: Client, web_hook_url: Option<String>) -> Self {
        DiscordWebhookClient {
            client,
            web_hook_url,
        }
    }

    pub async fn send_message(
        &self,
        username: &str,
        content: &str,
        embeds: Vec<DiscordWebhookEmbed>,
    ) -> Result<(), ReqwestError> {
        if let Some(url) = &self.web_hook_url {
            let webhook_payload = DiscordWebhookMessage {
                username: username.to_string(),
                content: if content.len() > 2000 {
                    content[..2000].to_string()
                } else {
                    content.to_string()
                },
                embeds,
            };

            self.client
                .post(url)
                .header(header::CONTENT_TYPE, "application/json")
                .json(&webhook_payload)
                .send()
                .await?;
        }

        Ok(())
    }

    pub async fn send_error_log(&self, error: &str) -> Result<(), ReqwestError> {
        self.send_message(
            "Gemini Assistant Server Error Log",
            &format!("🚨 **ERROR** 🚨\n```{}```", error),
            vec![],
        )
        .await
    }

    pub async fn send_token_info(
        &self,
        language: &str,
        total_tokens: i32,
        cleanded_error_output: &str,
    ) -> Result<(), ReqwestError> {
        self.send_message(
            "Gemini Assistant Server Log",
            &if cleanded_error_output.len() > 2000 {
                cleanded_error_output[..2000].to_string()
            } else {
                cleanded_error_output.to_string()
            },
            vec![DiscordWebhookEmbed {
                fields: vec![
                    DiscordWebhookField {
                        name: "language".to_string(),
                        value: language.to_string(),
                    },
                    DiscordWebhookField {
                        name: "tokens".to_string(),
                        value: total_tokens.to_string(),
                    },
                ],
            }],
        )
        .await
    }
}

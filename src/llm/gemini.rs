//! Gemini API client implementation

use anyhow::{Result, Context};
use ureq::{Agent, AgentBuilder};
use serde_json::json;
use std::env;
use std::time::Duration;

use crate::llm::schema::{LlmRequest, SystemInstruction, Content, Part, GenerationConfig, LlmResponse, AnimationCode};
use crate::llm::{parse_animation_code, LlmClient};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const MODEL: &str = "gemini-3.1-flash-lite";
const SYSTEM_PROMPT: &str = include_str!("../../prompts/system_prompt.txt");

/// Gemini API client
pub struct GeminiClient {
    agent: Agent,
    api_key: String,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY environment variable not set")?;

        let agent = AgentBuilder::new()
            .timeout(Duration::from_secs(60))
            .build();

        Ok(Self { agent, api_key })
    }

    /// Generate animation code from prompt
    pub async fn generate_animation(&self, prompt: &str, duration: u32) -> Result<AnimationCode> {
        let system_instruction = SystemInstruction {
            parts: vec![Part { text: SYSTEM_PROMPT.to_string() }],
        };

        let user_prompt = format!(
            "Create a {}-second animation: {}. Output valid JSON only with html, css, js fields.",
            duration, prompt
        );

        let request = LlmRequest {
            model: MODEL.to_string(),
            system_instruction,
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part { text: user_prompt }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
                response_schema: json!({
                    "type": "OBJECT",
                    "properties": {
                        "html": { "type": "STRING" },
                        "css": { "type": "STRING" },
                        "js": { "type": "STRING" }
                    },
                    "required": ["html", "css", "js"]
                }),
                temperature: 0.2,
            },
        };

        let url = format!("{}/{}:generateContent", GEMINI_API_BASE, MODEL);

        // ureq is synchronous, so we run it in a blocking task
        let response = tokio::task::spawn_blocking({
            let agent = self.agent.clone();
            let url = url.clone();
            let request = request.clone();
            let api_key = self.api_key.clone();
            move || {
                agent
                    .post(&url)
                    .set("x-goog-api-key", &api_key)
                    .send_json(&request)
            }
        }).await
        .context("Failed to execute blocking request")?
        .context("Failed to send request to Gemini API")?;

        let status = response.status();
        if !(200..300).contains(&status) {
            let error_text = response.into_string().unwrap_or_default();
            anyhow::bail!("Gemini API error (status {}): {}", status, error_text);
        }

        let llm_response: LlmResponse = response.into_json()
            .context("Failed to parse Gemini response")?;

        let candidate = llm_response.candidates.first()
            .context("No candidates in Gemini response")?;

        let text = candidate.content.parts.first()
            .context("No content parts in Gemini response")?
            .text.clone();

        // Parse the JSON from the response text
        parse_animation_code(&text)
    }
}

#[async_trait::async_trait]
impl LlmClient for GeminiClient {
    async fn generate_animation(&self, prompt: &str, duration: u32) -> Result<AnimationCode> {
        // Delegate to the inherent method so both call paths share one implementation.
        GeminiClient::generate_animation(self, prompt, duration).await
    }
}
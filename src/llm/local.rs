//! Client for an OpenAI-compatible local inference server.
//!
//! Endpoint is configurable via `LOCAL_API_URL` (default `http://localhost:8081/v1`).
//! An optional `LOCAL_API_MODEL` overrides the model name sent in the request.
//!
//! The server streams Server-Sent Events. Chunks may carry either `content` or
//! `reasoning_content`; only `content` is part of the answer, so reasoning
//! tokens are discarded. Thinking is disabled in the request because some
//! reasoning-tuned models will emit an unbounded reasoning stream and never
//! produce an answer.

use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

use crate::llm::schema::AnimationCode;
use crate::llm::{parse_animation_code, LlmClient};

const DEFAULT_BASE_URL: &str = "http://localhost:8081/v1";
const SYSTEM_PROMPT: &str = include_str!("../../prompts/system_prompt.txt");
/// Local models are slow; allow a long generation window.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// OpenAI-compatible chat completions client for a local inference server.
pub struct LocalClient {
    agent: Agent,
    endpoint: String,
    model: Option<String>,
}

impl LocalClient {
    /// Build a client, resolving the endpoint from the environment.
    pub fn new() -> Result<Self> {
        let base = env::var("LOCAL_API_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let endpoint = chat_completions_url(&base);
        let model = env::var("LOCAL_API_MODEL").ok().filter(|m| !m.trim().is_empty());

        tracing::debug!("Using local inference endpoint {endpoint}");

        let agent = AgentBuilder::new()
            .timeout(REQUEST_TIMEOUT)
            .build();

        Ok(Self { agent, endpoint, model })
    }
}

/// Normalise a base URL into a full chat-completions URL.
///
/// Accepts a bare base (`http://host:8081/v1`), a base with a trailing slash,
/// or an already-complete chat-completions URL.
fn chat_completions_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        return trimmed.to_string();
    }
    format!("{trimmed}/chat/completions")
}

#[async_trait::async_trait]
impl LlmClient for LocalClient {
    async fn generate_animation(&self, prompt: &str, duration: u32) -> Result<AnimationCode> {
        let user_prompt =
            format!("Create a {duration}-second animation: {prompt}. Output raw JSON only.");

        let mut payload = json!({
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user",   "content": user_prompt },
            ],
            "stream": true,
            // Small reasoning models otherwise loop and never emit content.
            "chat_template_kwargs": { "enable_thinking": false },
        });
        if let Some(model) = &self.model {
            payload["model"] = json!(model);
        }

        let agent = self.agent.clone();
        let endpoint = self.endpoint.clone();

        // ureq is synchronous and SSE reads block, so keep it off the async worker.
        let body = tokio::task::spawn_blocking(move || -> Result<String> {
            let response = agent
                .post(&endpoint)
                .set("Content-Type", "application/json")
                .send_json(&payload)
                .context("Failed to send request to local inference server")?;

            let status = response.status();
            if !(200..300).contains(&status) {
                let detail = response.into_string().unwrap_or_default();
                anyhow::bail!("Local inference server returned status {status}: {detail}");
            }

            collect_sse_content(response.into_reader())
        })
        .await
        .context("Local inference task panicked")??;

        tracing::debug!("Local model returned {} chars", body.len());
        parse_animation_code(&body)
    }
}

/// Read an SSE stream and concatenate every `choices[0].delta.content` fragment.
///
/// `reasoning_content` fragments are ignored, and the `data: [DONE]` sentinel
/// ends the stream. Comments (`: ping`) and blank keep-alive lines are skipped.
fn collect_sse_content<R: std::io::Read>(reader: R) -> Result<String> {
    let reader = BufReader::new(reader);
    let mut out = String::new();

    for line in reader.lines() {
        let line = line.context("Failed to read SSE stream line")?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(':') {
            continue;
        }
        let Some(data) = trimmed.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            if data == "[DONE]" {
                break;
            }
            continue;
        }

        let chunk: serde_json::Value = match serde_json::from_str(data) {
            Ok(value) => value,
            // A partial or non-JSON keep-alive frame should not abort the stream.
            Err(_) => continue,
        };

        if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str() {
            out.push_str(content);
        }
    }

    if out.trim().is_empty() {
        anyhow::bail!("Local inference server returned no content");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_base_urls() {
        assert_eq!(
            chat_completions_url("http://h:8081/v1"),
            "http://h:8081/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("http://h:8081/v1/"),
            "http://h:8081/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("http://h:8081/v1/chat/completions"),
            "http://h:8081/v1/chat/completions"
        );
    }

    #[test]
    fn collects_content_and_skips_reasoning() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":null}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"thinking...\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"{\\\"html\\\":\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"\\\"<b>\\\",\"}}]}\n\n",
            "data: [DONE]\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"IGNORED\"}}]}\n\n",
        );
        let out = collect_sse_content(sse.as_bytes()).expect("should collect");
        assert_eq!(out, "{\"html\":\"<b>\",");
        assert!(!out.contains("IGNORED"));
    }

    #[test]
    fn errors_on_empty_stream() {
        assert!(collect_sse_content("data: [DONE]\n".as_bytes()).is_err());
    }
}
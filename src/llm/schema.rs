//! Request/response types for LLM API

use serde::{Deserialize, Serialize};

/// Structured animation code output from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCode {
    pub html: String,
    pub css: String,
    pub js: String,
}

/// LLM request payload
#[derive(Debug, Clone, Serialize)]
pub struct LlmRequest {
    pub model: String,
    pub system_instruction: SystemInstruction,
    pub contents: Vec<Content>,
    pub generation_config: GenerationConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInstruction {
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    pub response_mime_type: String,
    #[serde(rename = "responseSchema")]
    pub response_schema: serde_json::Value,
    pub temperature: f32,
}

/// LLM response
#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: ResponseContent,
}

#[derive(Debug, Deserialize)]
pub struct ResponseContent {
    pub parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
pub struct ResponsePart {
    pub text: String,
}
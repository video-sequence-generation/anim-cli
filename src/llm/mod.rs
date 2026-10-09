//! LLM integration layer

pub mod schema;
pub mod gemini;
pub mod local;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::llm::schema::AnimationCode;

pub use gemini::GeminiClient;
pub use local::LocalClient;

/// A source of animation code generated from a natural language prompt.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate animation code from a natural language prompt.
    async fn generate_animation(&self, prompt: &str, duration: u32) -> Result<AnimationCode>;
}

/// Parse an LLM response body into [`AnimationCode`].
///
/// Small models routinely wrap JSON in prose or a fenced code block, and
/// sometimes trail off after the closing brace. Try a strict parse first, then
/// fall back to extracting the first balanced top-level object.
pub(crate) fn parse_animation_code(text: &str) -> Result<AnimationCode> {
    match serde_json::from_str::<AnimationCode>(text.trim()) {
        Ok(code) => return Ok(code),
        Err(strict_err) => {
            let candidate = extract_json_object(text)
                .context("No balanced JSON object found in LLM response")?;
            serde_json::from_str::<AnimationCode>(candidate).map_err(|e| {
                anyhow::anyhow!(
                    "LLM response was not valid animation code ({strict_err}); \
                     extracted object also failed to parse: {e}"
                )
            })
        }
    }
}

/// Return the first balanced `{...}` span, ignoring braces inside JSON strings.
fn extract_json_object(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let start = bytes.iter().position(|b| *b == b'{')?;

    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (i, b) in bytes.iter().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        match b {
            b'\\' if in_string => escaped = true,
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_json() {
        let code = parse_animation_code(
            r#"{"html":"<div></div>","css":"body{}","js":"//x"}"#,
        )
        .expect("should parse");
        assert_eq!(code.html, "<div></div>");
    }

    #[test]
    fn parses_fenced_json() {
        let raw = "Here you go:\n```json\n{\"html\":\"<p>\",\"css\":\"\",\"js\":\"\"}\n```\nDone.";
        let code = parse_animation_code(raw).expect("should parse");
        assert_eq!(code.html, "<p>");
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let raw = r#"prose {"html":"<div>{not: 'a brace'}</div>","css":"","js":""} trailing"#;
        let code = parse_animation_code(raw).expect("should parse");
        assert_eq!(code.html, "<div>{not: 'a brace'}</div>");
    }

    #[test]
    fn errors_when_no_object_present() {
        assert!(parse_animation_code("no json at all").is_err());
    }
}
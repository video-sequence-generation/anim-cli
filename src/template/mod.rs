//! HTML template generation

use crate::llm::schema::AnimationCode;

/// HTML template for HyperFrames
pub struct HtmlTemplate {
    duration: u32,
}

impl HtmlTemplate {
    /// Create a new template with duration
    pub fn new(duration: u32) -> Self {
        Self { duration }
    }

    /// Render the template with animation code
    pub fn render(&self, code: &AnimationCode) -> String {
        format!(r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
{css}
  </style>
</head>
<body data-duration="{duration}s">
{html}
  <script>
{js}
    // HyperFrames auto-binds renderFrame to frame clock
  </script>
</body>
</html>"#,
            css = code.css,
            html = code.html,
            js = code.js,
            duration = self.duration,
        )
    }
}
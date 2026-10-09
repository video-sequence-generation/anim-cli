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
  <meta name="viewport" content="width=1920, height=1080">
  <script src="https://cdn.jsdelivr.net/npm/gsap@3.14.2/dist/gsap.min.js"></script>
  <style>
{css}
  </style>
</head>
<body>
  <div
    id="root"
    data-composition-id="main"
    data-start="0"
    data-width="1920"
    data-height="1080"
    data-duration="{duration}"
    style="position: relative; width: 1920px; height: 1080px; overflow: hidden;"
  >
{html}
  </div>
  <script>
{js}
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
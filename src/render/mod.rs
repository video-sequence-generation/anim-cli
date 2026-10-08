//! Render orchestration using HyperFrames

use anyhow::{Result, Context};
use tokio::process::Command;
use std::path::Path;

/// HyperFrames render engine wrapper
pub struct RenderEngine {
    fps: u32,
}

impl RenderEngine {
    /// Create a new render engine
    pub fn new(fps: u32) -> Self {
        Self { fps }
    }

    /// Render HTML to MP4 video
    pub async fn render(&self, html_path: &Path, output_path: &str) -> Result<()> {
        tracing::info!("Rendering {} to {} at {}fps", html_path.display(), output_path, self.fps);

        // Ensure hyperframes CLI is available
        self.ensure_hyperframes().await?;

        let output = Command::new("npx")
            .args([
                "hyperframes",
                "render",
                html_path.to_str().unwrap(),
                "-o", output_path,
                "--fps", &self.fps.to_string(),
            ])
            .output()
            .await
            .context("Failed to execute hyperframes render command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("HyperFrames render failed: {}", stderr);
        }

        tracing::debug!("HyperFrames stdout: {}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    /// Preview HTML in browser
    pub async fn preview(&self, html_path: &Path) -> Result<()> {
        tracing::info!("Opening preview for {}", html_path.display());

        self.ensure_hyperframes().await?;

        let mut cmd = Command::new("npx");
        cmd.args([
            "hyperframes",
            "preview",
            html_path.to_str().unwrap(),
        ]);

        // Spawn and detach for preview
        cmd.spawn()
            .context("Failed to start hyperframes preview")?;

        tracing::info!("Preview started in browser");
        Ok(())
    }

    /// Ensure hyperframes CLI is available
    async fn ensure_hyperframes(&self) -> Result<()> {
        let output = Command::new("npx")
            .args(["hyperframes", "--version"])
            .output()
            .await
            .context("Failed to check hyperframes version")?;

        if !output.status.success() {
            anyhow::bail!("HyperFrames CLI not found. Install with: npm install -g @hyperframes/cli");
        }

        Ok(())
    }
}
//! CLI entry point for anim-cli

use clap::{Parser, Subcommand};
use anyhow::Result;

mod llm;
mod render;
mod template;
mod utils;

use llm::GeminiClient;
use render::RenderEngine;
use template::HtmlTemplate;
use utils::{TempDir, setup_logging};

#[derive(Parser)]
#[command(name = "anim-cli")]
#[command(about = "Convert natural language prompts to 60 FPS MP4 videos using HyperFrames + LLMs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a prompt to video
    Render {
        /// Natural language prompt describing the animation
        prompt: String,

        /// Duration in seconds
        #[arg(short, long, default_value = "5")]
        duration: u32,

        /// Output file path
        #[arg(short, long, default_value = "output.mp4")]
        output: String,

        /// Frames per second
        #[arg(short, long, default_value = "60")]
        fps: u32,

        /// Preview in browser instead of rendering to video
        #[arg(long)]
        preview: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Render { prompt, duration, output, fps, preview } => {
            run_render(prompt, duration, output, fps, preview).await
        }
    }
}

async fn run_render(prompt: String, duration: u32, output: String, fps: u32, preview: bool) -> Result<()> {
    tracing::info!("Starting render: \"{}\" ({}s @ {}fps)", prompt, duration, fps);

    // 1. Initialize LLM client
    let llm_client = GeminiClient::new()?;

    // 2. Generate animation code from prompt
    let animation_code = llm_client.generate_animation(&prompt, duration).await?;
    tracing::debug!("Generated animation code: {} chars HTML, {} chars CSS, {} chars JS",
        animation_code.html.len(), animation_code.css.len(), animation_code.js.len());

    // 3. Create temp directory and write HTML file
    let temp_dir = TempDir::new()?;
    let html_path = temp_dir.path().join("index.html");

    let html_template = HtmlTemplate::new(duration);
    let html_content = html_template.render(&animation_code);
    std::fs::write(&html_path, html_content)?;
    tracing::info!("Written HTML to {}", html_path.display());

    // 4. Render using HyperFrames
    let render_engine = RenderEngine::new(fps);
    if preview {
        render_engine.preview(&html_path).await?;
    } else {
        render_engine.render(&html_path, &output).await?;
        tracing::info!("Video saved to {}", output);
    }

    // 5. Cleanup (TempDir drops automatically)
    Ok(())
}
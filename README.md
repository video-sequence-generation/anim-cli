# anim-cli

Lightweight Animation CLI - Convert natural language prompts to 60 FPS MP4 videos using HyperFrames + LLMs.

## Features

- **Natural language to video**: Describe animations in plain English
- **60 FPS output**: High-quality MP4 videos via HyperFrames + headless Chromium
- **Cross-platform**: Single binary distribution (Rust)
- **Multiple LLM providers**: Gemini, Claude, GPT-4o (extensible)
- **Preview mode**: Instant browser preview before rendering

## Installation

### From Source

```bash
git clone https://github.com/video-sequence-generation/anim-cli
cd anim-cli
cargo install --path .
```

### Pre-built Binaries

Download from [Releases](https://github.com/video-sequence-generation/anim-cli/releases).

## Usage

```bash
# Set your API key
export GEMINI_API_KEY="your-api-key"

# Render a 6-second animation
anim-cli render "Neon cyberpunk radar sweep" --duration 6 --output radar.mp4

# Preview in browser instead of rendering
anim-cli render "Glowing particle system" --preview

# Custom FPS
anim-cli render "Smooth wave animation" --fps 30 --output wave.mp4
```

## Requirements

- **Node.js** (for HyperFrames CLI): `npm install -g @hyperframes/cli`
- **FFmpeg** (for video encoding)
- **LLM API Key**: Set `GEMINI_API_KEY` environment variable

## Architecture

```
User Prompt → LLM (Gemini) → HTML/CSS/JS → HyperFrames → MP4
```

- **CLI**: Rust (clap, tokio)
- **LLM Client**: ureq + serde (no OpenSSL dependency)
- **Render Engine**: HyperFrames (npx wrapper)
- **Template**: Frame-deterministic HTML with data-* attributes

## Animation Rules (System Prompt)

The LLM generates frame-deterministic animations following these rules:

1. **NO real-time drivers**: No `requestAnimationFrame`, `setInterval`, `setTimeout`, `performance.now()`
2. **Frame-deterministic**: All animation driven by `progress` parameter (0.0 to 1.0)
3. **Mathematical motion**: Use `Math.sin`, `Math.cos`, easing functions
4. **Fixed viewport**: 1920×1080, centered content
5. **No external dependencies**: Pure HTML5, CSS3, ES6
6. **HyperFrames attributes**: `data-duration` on root, `data-start`/`data-duration` on elements
7. **JS interface**: Export `function renderFrame(container, progress) { ... }`

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## License

MIT
//! Utility functions

use anyhow::{Result, Context};
use tempfile::TempDir as TempfileTempDir;
use std::path::Path;

/// Wrapper around tempfile::TempDir with custom path
pub struct TempDir {
    inner: TempfileTempDir,
}

impl TempDir {
    /// Create a new temporary directory
    pub fn new() -> Result<Self> {
        let inner = TempfileTempDir::new()
            .context("Failed to create temporary directory")?;
        Ok(Self { inner })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.inner.path()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // TempDir automatically cleans up on drop
    }
}

/// Setup logging
pub fn setup_logging() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "filedock", about = "Remote file management service")]
pub struct Config {
    /// Data directory to serve (default: ./file_dock_data)
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,

    /// Server host
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: String,

    /// Web UI port
    #[arg(short, long, default_value_t = 18888)]
    pub port: u16,

    /// WebDAV port (separate from web UI)
    #[arg(short, long, default_value_t = 17777)]
    pub webdav_port: u16,
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    /// Get the data directory, creating it if it doesn't exist
    pub fn get_data_dir(&self) -> anyhow::Result<PathBuf> {
        let data_dir = match &self.data_dir {
            Some(dir) => dir.clone(),
            None => {
                // Use current directory / file_dock_data
                let current_dir = std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."));
                current_dir.join("file_dock_data")
            }
        };

        // Create directory if it doesn't exist
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
            tracing::info!("Created data directory: {}", data_dir.display());
        }

        // Verify it's a directory
        if !data_dir.is_dir() {
            anyhow::bail!("Data path is not a directory: {}", data_dir.display());
        }

        Ok(data_dir)
    }

    pub fn web_bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn webdav_bind_address(&self) -> String {
        format!("{}:{}", self.host, self.webdav_port)
    }
}

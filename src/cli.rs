use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "llama-monitor",
    about = "Web dashboard for llama.cpp server management and GPU monitoring"
)]
pub struct AppArgs {
    /// Path to the llama-server binary
    #[arg(short = 's', long)]
    pub llama_server_path: Option<PathBuf>,

    /// Working directory for llama-server
    #[arg(long)]
    pub llama_server_cwd: Option<PathBuf>,

    /// Port for the monitor web UI
    #[arg(short, long, default_value_t = 7778)]
    pub port: u16,

    /// Directory containing .gguf model files for auto-discovery
    #[arg(short = 'm', long)]
    pub models_dir: Option<PathBuf>,

    /// Path to presets JSON file
    #[arg(long)]
    pub presets_file: Option<PathBuf>,

    /// GPU monitoring backend: auto (all detected vendors), rocm, nvidia, none
    #[arg(long, default_value = "auto")]
    pub gpu_backend: String,

    /// GPU architecture for ROCm env (e.g. gfx906, gfx1100, auto)
    #[arg(long)]
    pub gpu_arch: Option<String>,

    /// Visible GPU device indices (e.g. 0,1,2,3)
    #[arg(long)]
    pub gpu_devices: Option<String>,
}

use std::path::PathBuf;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Meta {
    // path of config file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub data_path: Option<PathBuf>,

    #[arg(short, long)]
    pub library: Option<Vec<String>>,

    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    #[arg(short, long, default_value = "3177")]
    pub port: u16,

    #[arg(long, default_value = "")]
    pub public_url: String,

    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    Serve,
}

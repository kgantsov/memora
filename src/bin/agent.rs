use memora::agent::agent::Agent;

use clap::Parser;
use std::path::{PathBuf};

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory where files are stored
    #[arg(short, long, default_value = "./data")]
    dir: PathBuf,

    /// Token for authentication
    #[arg(short = 't', long)]
    token: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if !args.dir.is_dir() {
        eprintln!("The specified directory does not exist: {:?}", args.dir);
        std::process::exit(1);
    }

    let agent = Agent::new(args.token, args.dir);
    agent.run_scanner().await;
}
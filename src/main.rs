mod repository;
mod utilities;
mod object;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::repository::GitRepository;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    Init {
        path: Option<PathBuf>,
    },
}


fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Command::Init { path} => {
            let mut p = PathBuf::new();
            if let Some(path) = path.clone() {
                p =  path.clone();
            } else {
                if let Ok(path) = std::env::current_dir() {
                    p = path;
                }
            }
            GitRepository::create_new_repo(p);
        }
    }
}
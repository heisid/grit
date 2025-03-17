mod repository;
mod utilities;
mod object;

use crate::repository::GitRepository;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    CatFile {
        #[arg(
            value_name="TYPE",
            value_parser = clap::builder::PossibleValuesParser::new(
                ["blob", "commit", "tag", "tree"])
        )]
        object_type: String,
        sha: String,
    },
    HashObject {
        #[arg(short, long)]
        write: bool,
        #[arg(
            short,
            long,
            value_name = "TYPE",
            value_parser = clap::builder::PossibleValuesParser::new(
            ["blob", "commit", "tag", "tree"])
        )]
        object_type: String,
        #[arg(value_name = "FILE")]
        file_path: PathBuf,
    },
    Log,
    Commit {
        message: String,
    },
    Checkout {
        branch: String,
    },
    Rm {
        pathspec: PathBuf,
    },
    Status,
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
        _ => {}
    }
}
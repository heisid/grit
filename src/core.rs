use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use ini::ini;

pub struct GitRepository {
    pub worktree: PathBuf,
    gitdir: PathBuf,
    conf: HashMap<String, HashMap<String, Option<String>>>
}

fn die(msg: &str) {
    println!("{}", msg);
    process::exit(1);
}

impl GitRepository {
    pub fn new(worktree: PathBuf) -> Self {
        let gitdir = worktree.join(".git");
        let conf: HashMap<String, HashMap<String, Option<String>>> = HashMap::new();
        Self { worktree, gitdir, conf }
    }

    pub fn from_dir(worktree: PathBuf) -> Self {
        if !worktree.exists() {
            die("Worktree does not exist");
        }
        let gitdir = worktree.join(".git");
        if !gitdir.exists() {
            die("Not a valid git repository");
        }
        let conf_file = gitdir.join("config");
        if !conf_file.exists() {
            die("Config file does not exist");
        }
        let conf = ini!(conf_file.to_str().unwrap());
        let repo_format_version = conf["core"]["repositoryformatversion"].clone();
        if let Some(version) = repo_format_version {
            if version != "0" {
                die("Unsupported format version");
            }
        } else {
            die("Could not determine repository format version");
        }
        
        Self {
            worktree,
            gitdir,
            conf
        }
    }
}
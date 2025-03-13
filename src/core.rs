use crate::utilities::{die, ConfigHashMap};
use ini::ini;
use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::PathBuf;
use configparser::ini::Ini;

pub struct GitRepository {
    pub worktree: PathBuf,
    gitdir: PathBuf,
    conf: HashMap<String, HashMap<String, Option<String>>>
}

impl GitRepository {
    pub fn new(worktree: PathBuf) -> Self {
        let gitdir = worktree.join(".git");
        let conf: ConfigHashMap = HashMap::new();
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

    pub fn create_new_repo(worktree: PathBuf) {
        let repo = GitRepository::new(worktree);
        if repo.worktree.exists() {
            if repo.gitdir.exists() {
                die("Repo already exists");
            }
        }

        let create_gitdir = create_dir(repo.gitdir.clone());
        if let Err(_) = create_gitdir {
            die("Failed to initialize repository");
        }

        if let Err(_) = create_dir(repo.gitdir.clone().join("objects")) {
            die("Failed to create objects directory");
        }

        if let Err(_) = create_dir(repo.gitdir.clone().join("refs").join("heads")) {
            die("Failed to create git refs/head directory");
        }

        if let Err(_) = create_dir(repo.gitdir.clone().join("refs").join("tags")) {
            die("Failed to create git refs/tags directory");
        }

        if let Ok(mut head_file) = File::create(repo.gitdir.clone().join("HEAD")) {
            if let Err(_) = head_file.write_all(&"ref: refs/heads/master\n".as_ref()) {
                die("Failed to write HEAD file");
            }
        } else {
            die("Failed to create HEAD file");
        }

        let mut config = Ini::new();
        config.set("core", "repositoryformatversion", Some("0".to_owned()));
        config.set("core", "filemode", Some("false".to_owned()));
        config.set("core", "bare", Some("false".to_owned()));

        if let Err(_) = config.write(repo.gitdir.join("config")) {
            die("Failed to write config file");
        }


        if let Ok(mut description_file)  = File::create(repo.gitdir.clone().join("config")) {
            if let Err(_) = description_file.write("Unnamed repository; edit this file 'description' to name the repository.\n".as_ref()) {
                die("Failed to write description file");
            }
        }
    }
}
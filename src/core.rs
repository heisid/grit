use crate::utilities::ConfigHashMap;
use crate::{create_path_or_die, die};
use configparser::ini::Ini;
use ini::ini;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

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
            die!("Worktree does not exist");
        }
        let gitdir = worktree.join(".git");
        if !gitdir.exists() {
            die!("Not a valid git repository");
        }
        let conf_file = gitdir.join("config");
        if !conf_file.exists() {
            die!("Config file does not exist");
        }
        let conf = ini!(conf_file.to_str().unwrap());
        let repo_format_version = conf["core"]["repositoryformatversion"].clone();
        if let Some(version) = repo_format_version {
            if version != "0" {
                die!("Unsupported format version");
            }
        } else {
            die!("Could not determine repository format version");
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
                die!("Repo already exists");
            }
        }

        create_path_or_die!(dir: repo.gitdir.clone(), "Failed to initialize repository");

        create_path_or_die!(dir: repo.gitdir.clone().join("objects"), "Failed to create objects directory");

        create_path_or_die!(dir: repo.gitdir.clone().join("refs").join("heads"), "Failed to create git refs/head directory");

        create_path_or_die!(dir: repo.gitdir.clone().join("refs").join("tags"), "Failed to create git refs/tags directory");

        create_path_or_die!(file: repo.gitdir.clone().join("HEAD"), &"ref: refs/heads/master\n", "Failed to write HEAD file");

        let mut config = Ini::new();
        config.set("core", "repositoryformatversion", Some("0".to_owned()));
        config.set("core", "filemode", Some("false".to_owned()));
        config.set("core", "bare", Some("false".to_owned()));

        if let Err(_) = config.write(repo.gitdir.join("config")) {
            die!("Failed to write config file");
        }

        create_path_or_die!(
            file: repo.gitdir.clone().join("HEAD"),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
            "Failed to write description file");
    }
}
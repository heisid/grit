use crate::utilities::{path_should_exist, path_should_not_exist, ConfigHashMap};
use crate::{create_path_or_die, die};
use configparser::ini::Ini;
use ini::ini;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

pub struct GitRepository {
    pub worktree: PathBuf,
    gitdir: PathBuf,
    conf: ConfigHashMap
}

impl GitRepository {
    pub fn new(worktree: PathBuf) -> Self {
        let gitdir = worktree.join(".git");
        let conf: ConfigHashMap = HashMap::new();
        Self { worktree, gitdir, conf }
    }

    pub fn from_dir(worktree: PathBuf) -> Self {
        path_should_exist(&worktree, "Worktree does not exist");
        let gitdir = worktree.join(".git");
        path_should_exist(&gitdir, "Not a valid git repository");
        let conf_file = gitdir.join("config");
        path_should_exist(&conf_file, "Configuration file not found");
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
        path_should_not_exist(&repo.gitdir, "Already a git repository");

        if !repo.worktree.exists() {
            create_path_or_die!(dir: repo.worktree.clone(), "Could not create worktree");
        }
        create_path_or_die!(dir: repo.gitdir.clone(), "Failed to initialize repository");
        create_path_or_die!(dir: repo.gitdir.clone().join("objects"), "Failed to create objects directory");
        create_path_or_die!(dir: repo.gitdir.clone().join("refs"), "Failed to create refs directory");
        create_path_or_die!(dir: repo.gitdir.clone().join("refs").join("heads"), "Failed to create git refs/head directory");
        create_path_or_die!(dir: repo.gitdir.clone().join("refs").join("tags"), "Failed to create git refs/tags directory");
        create_path_or_die!(file: repo.gitdir.clone().join("HEAD"), "ref: refs/heads/master\n", "Failed to write HEAD file");
        create_path_or_die!(
            file: repo.gitdir.clone().join("HEAD"),
            "ref: refs/heads/master\n",
            "Failed to write description file");

        let mut config = Ini::new();
        config.set("core", "repositoryformatversion", Some("0".to_owned()));
        config.set("core", "filemode", Some("false".to_owned()));
        config.set("core", "bare", Some("false".to_owned()));

        if let Err(_) = config.write(repo.gitdir.join("config")) {
            die!("Failed to write config file");
        }
        println!("Git repository initialized in {}", repo.worktree.display());
    }
}
use std::collections::HashMap;
use std::path::PathBuf;

pub type ConfigHashMap = HashMap<String, HashMap<String, Option<String>>>;

#[macro_export]
macro_rules! die {
    ($message:expr) => {
        println!("{}", $message);
        std::process::exit(1);
    }
}

#[macro_export]
macro_rules! create_path_or_die {
    (dir: $path:expr, $message:expr) => {
        if let Err(e) =  std::fs::create_dir($path) {
            die!(format!("{}\nError: {}", $message, e.to_string()));
        }
    };

    (file: $path:expr, $content:expr, $message:expr) => {
        if let Ok(mut file) = std::fs::File::create($path) {
            if let Err(e) = file.write_all($content.as_ref()) {
                die!(format!("{}\nError: {}", $message, e.to_string()));
            }
        } else {
            die!($message);
        }
    };
}

pub fn path_should_exist(path: &PathBuf, message: &str) {
    if !path.exists() {
        die!(message);
    }
}

pub fn path_should_not_exist(path: &PathBuf, message: &str) {
    if path.exists() {
        die!(message);
    }
}

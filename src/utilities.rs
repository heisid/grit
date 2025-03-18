use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::PathBuf;

pub type ConfigHashMap = HashMap<String, HashMap<String, Option<String>>>;

#[macro_export]
macro_rules! die {
    ($msg:expr) => {
        {
            println!("{}", $msg);
            std::process::exit(1);
        }
    };
    ($fmt:expr, $($args:tt)*) => {
        {
            println!($fmt, $($args)*);
            std::process::exit(1);
        }
    };
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
            if let Err(e) = write!(file, $content) {
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

pub fn deserialize_kv_with_message(data: &Vec<u8>) -> (IndexMap<String, String>, String) {
    let string_rep = String::from_utf8(data.clone());
    if string_rep.is_err() {
        die!("Failed to parse commit data");
    }
    let string_rep = string_rep.unwrap();
    let mut is_message = false;
    let mut header: IndexMap<String, String> = IndexMap::new();
    let mut message = String::new();
    let mut last_key: Option<String> = None;
    for line in string_rep.lines() {
        if is_message {
            message.push_str(format!("\n{}", line).as_str());
            continue;
        }

        if line == "" {
            is_message = true;
            continue;
        }

        // continuation of previous value (multiline value)
        if line.starts_with(" ") && last_key.is_some() {
            let key = last_key.clone().unwrap();
            let line = &line[1..]; // trim first space
            match header.get(&key) {
                Some(val) => {
                    let mut new_val = val.clone();
                    new_val.push_str(format!("\n{}", line).as_str());
                    header.insert(key, new_val);
                }
                None => {
                    header.insert(key, line.to_string());
                }
            }
        } else {
            let mut line_iter = line.splitn(2, ' ');
            let key = line_iter.next().unwrap();
            let val = line_iter.next().unwrap();
            header.insert(key.to_string(), val.to_string());
            last_key = Some(key.to_string());
        }
    }
    (header, message)
}

pub fn serialize_kv_with_message(header: &IndexMap<String, String>, message: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for (key, val) in header.into_iter() {
        let val = val.replace("\n ", "\n");
        result.extend(key.as_bytes());
        result.extend(val.as_bytes());
    }
    result.push(b'\n');
    result.extend(message.as_bytes());
    result
}

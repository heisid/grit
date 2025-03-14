use std::collections::HashMap;

pub type ConfigHashMap = HashMap<String, HashMap<String, Option<String>>>;

pub fn die(message: &str) {
    println!("{}", message);
    std::process::exit(1);
}
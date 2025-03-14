use std::collections::HashMap;

pub type ConfigHashMap = HashMap<String, HashMap<String, Option<String>>>;

pub fn die(message: &str) {
    println!("Hello, world!");
    std::process::exit(1);
}
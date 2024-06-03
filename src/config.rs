//use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Interface {
    device: String,
    ip: String,
    mask: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouterConfig {
    interface: Vec<Interface>,
}

impl RouterConfig {
    pub fn from_file(filename: &str) -> Self {
        let content = fs::read_to_string(filename).expect("Failed to read file");
        serde_yaml::from_str(&content).expect("Failed to parse YAML")
    }
}

use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Interface {
    pub device: String,
    pub ip: String,
    pub mask: u8,
}

#[derive(Debug, Deserialize)]
pub struct RouterConfig {
    pub interface: Vec<Interface>,
}

impl RouterConfig {
    pub fn from_file(filename: &str) -> Self {
        let content = fs::read_to_string(filename).expect("Failed to read file");
        serde_yaml::from_str(&content).expect("Failed to parse YAML")
    }
}

use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Interface {
    pub device: String,
    pub ip: String,
    pub mask: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InterfaceWrapper {
    #[serde(rename = "interface")]
    pub interface: Interface,
}

impl InterfaceWrapper {
    pub fn from_yaml(file_path: &str) -> Vec<Self> {
        let content = fs::read_to_string(file_path).expect("Failed to read the file");
        serde_yaml::from_str(&content).expect("Failed to parse YAML")
    }
}
use std::io::{self, Read, Write, Result};
use std::net::{TcpListener, TcpStream};
use std::fs;
use serde_yaml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipEntry {
    pub address_family: u16,
    pub route_tag: u16,
    pub ip_address: String,
    pub subnet_mask: String,
    pub next_hop: String,
    pub metric: u32,
}

lazy_static! {
    pub static ref ROUTING_TABLE: Mutex<HashMap<String, RipEntry>> = Mutex::new(HashMap::new());
}


#[derive(Debug, Serialize, Deserialize)]
struct Interface {
    device: String,
    ip: String,
    mask: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    interface: Interface,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RipMessage {
    pub command: u8,
    pub version: u8,
    pub must_be_zero: u16,
    pub entries: Vec<RipEntry>,
}

fn convert_to_rip(data: Vec<Data>) -> RipMessage {
    let entries: Vec<RipEntry> = data.into_iter().map(|d| {
        RipEntry {
            address_family: 2, // IP
            route_tag: 0, // Typically 0 unless specified
            ip_address: d.interface.ip.clone(),
            subnet_mask: format!("255.255.255.{}", 256 - 2_u32.pow(32 - d.interface.mask)), // Simplistic conversion, consider better handling
            next_hop: "0.0.0.0".to_string(), // Convert to String; Directly connected
            metric: 1, // Direct route
        }
    }).collect();

    RipMessage {
        command: 2, // Response
        version: 2, // RIP version 2
        must_be_zero: 0,
        entries,
    }
}

fn load_yaml_from_file(router: &str) -> io::Result<Vec<Data>> {
    println!("Loading YAML file for router: {}", router);
    let file_path = format!("config/routeur-{}.yaml", router);
    let yaml_content = fs::read_to_string(file_path)?;

    let data: Vec<Data> = serde_yaml::from_str(&yaml_content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    Ok(data)
}

fn update_routing_table_from_rip(rip_message: &RipMessage) {
    let mut routing_table = ROUTING_TABLE.lock().unwrap();
    for entry in &rip_message.entries {
        let key = format!("{}/{}", entry.ip_address, entry.subnet_mask);
        if !routing_table.contains_key(&key) || routing_table[&key].metric > entry.metric {
            routing_table.insert(key, entry.clone());
        }
    }
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let mut total_data = String::new();
    while let Ok(nbytes) = stream.read(&mut buffer) {
        if nbytes == 0 {
            break;
        }
        total_data.push_str(&String::from_utf8_lossy(&buffer[..nbytes]));
        if total_data.ends_with("#END#") {
            let clean_data = total_data.trim_end_matches("#END#");
            println!("Received data: {}", clean_data);
            if let Ok(rip_message) = serde_yaml::from_str::<RipMessage>(clean_data) {
                
                update_routing_table_from_rip(&rip_message);
            }
            total_data.clear();
        }
    }
    Ok(())
}


pub fn run_server(address: &str,router: &str) -> Result<()> {
    // Chargement initial des données de routage
    if let Ok(data) = load_yaml_from_file(router) {
        let rip_message = convert_to_rip(data);
        update_routing_table_from_rip(&rip_message);
    }

    let listener = TcpListener::bind(address)?;
    println!("Server listening on {}", address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr()?);
                handle_client(stream)?;
            },
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}


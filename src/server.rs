use std::io::{self, Read, Write, Result};
use std::net::{TcpListener, TcpStream};
use std::fs;
use serde_yaml;
use serde::{Deserialize, Serialize};

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
pub struct RipEntry {
    pub address_family: u16,
    pub route_tag: u16,
    pub ip_address: String,
    pub subnet_mask: String,
    pub next_hop: String,
    pub metric: u32,
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

fn load_yaml_from_file() -> io::Result<Vec<Data>> {
    let yaml_content = fs::read_to_string("config/routeur-r1.yaml")?;
    let data: Vec<Data> = serde_yaml::from_str(&yaml_content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(data)
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    while let Ok(nbytes) = stream.read(&mut buffer) {
        if nbytes == 0 {
            break; // Connection closed
        }
        let received_message = String::from_utf8_lossy(&buffer[..nbytes]).trim().to_string();
        if received_message == "Hello, server!" {
            match load_yaml_from_file() {
                Ok(data) => {
                    let rip_message = convert_to_rip(data);
                    let mut response = serde_yaml::to_string(&rip_message)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    
                    response.push_str("#END#");

                    println!("Sending response: {}", response);
                    stream.write_all(response.as_bytes())?;
                },
                Err(err) => {
                    eprintln!("Error loading or parsing YAML: {}", err);
                    let error_message = "Error loading YAML#END#";
                    stream.write_all(error_message.as_bytes())?;
                }
            }
        }
        stream.flush()?;
    }
    Ok(())
}


pub fn run_server(address: &str) -> Result<()> {
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

use std::io::{self, Read, Write};
use std::net::TcpStream;
use crate::server::ROUTING_TABLE;
use crate::server::RipMessage;

pub fn run_client(address: &str, message: &str) -> io::Result<String> {
    println!("Connecting to server at {}", address);
    let mut stream = TcpStream::connect(address)?;
    println!("Connected to server, sending message...");
    stream.write_all(message.as_bytes())?;

    let mut response = String::new();
    let mut buffer = [0; 1024];
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        response.push_str(&String::from_utf8_lossy(&buffer[..n]));
        if response.ends_with("#END#") {
            println!("Complete message received.");
            break;
        }
    }

    let clean_response = response.trim_end_matches("#END#").to_string();
    Ok(clean_response)
}


pub fn send_routing_update(address: &str) -> io::Result<()> {
    println!("Attempting to send routing update to {}", address);
    let routing_table = ROUTING_TABLE.lock().unwrap();
    let rip_message = RipMessage {
        command: 2,
        version: 2,
        must_be_zero: 0,
        entries: routing_table.values().cloned().collect(),
    };

    match serde_yaml::to_string(&rip_message) {
        Ok(serialized) => {
            let full_message = serialized + "#END#";
            match run_client(address, &full_message) {
                Ok(_) => println!("Routing update sent successfully."),
                Err(e) => {
                    println!("Error sending routing update: {}", e);
                    return Err(e);
                }
            }
        },
        Err(e) => {
            println!("Error serializing routing update: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    }
    Ok(())
}


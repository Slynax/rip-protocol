mod client;
mod server;

use std::thread;
use std::time::Duration;

use crate::client::run_client;
use crate::server::run_server;


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use serde_yaml;

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
            let server = thread::spawn(|| {
                run_server("127.0.0.1:65433").unwrap();
            });
            thread::sleep(Duration::from_millis(100)); // Ensure the server is ready
        });
    }

    #[test]
    fn test_response_from_server() {
        initialize();

        let client = thread::spawn(|| {
            let response = run_client("127.0.0.1:65433", "Hello, server!").unwrap();
            let rip_message: server::RipMessage = serde_yaml::from_str(&response).expect("Failed to parse YAML");
            

            println!("{:#?}", rip_message);
            assert_eq!(rip_message.version, 2);
            assert_eq!(rip_message.command, 2);
            assert_eq!(rip_message.entries[0].ip_address, "192.168.1.254"); // Example check
        });

        client.join().unwrap();
    }
}


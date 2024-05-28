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

    fn setup_server(address: String) {
        thread::spawn(move || {
            run_server(&address).expect("Server failed to run");
        });
    }
    

    fn initialize() {
        INIT.call_once(|| {
            setup_server("127.0.0.1:65433".to_string());
            thread::sleep(Duration::from_millis(100)); // Ensure the server is ready
        });
    }

    // Ancien test pour une seule paire client/serveur
    #[test]
    fn test_response_from_server() {
        initialize();

        let client = thread::spawn(|| {
            let response = run_client("127.0.0.1:65433", "Hello, server!").unwrap();
            let rip_message: server::RipMessage = serde_yaml::from_str(&response).expect("Failed to parse YAML");

            assert_eq!(rip_message.version, 2);
            assert_eq!(rip_message.command, 2);
            assert_eq!(rip_message.entries[0].ip_address, "192.168.1.254");
            assert_eq!(rip_message.entries[0].subnet_mask, "255.255.255.0");
            assert_eq!(rip_message.entries[1].ip_address, "10.1.1.1");
            assert_eq!(rip_message.entries[1].subnet_mask, "255.255.255.252");
        });

        client.join().unwrap();
    }

    // Nouveau test pour deux paires client/serveur
    #[test]
    fn test_dual_client_server_communication() {
        // Initialiser deux serveurs sur des ports différents
        setup_server("127.0.0.1:65433".to_string());
        setup_server("127.0.0.1:65434".to_string());


        // Donner un peu de temps pour que les serveurs démarrent
        thread::sleep(Duration::from_secs(1));

        // Créer des handles pour les clients
        let client_handle_one = thread::spawn(|| {
            let response = run_client("127.0.0.1:65433", "Hello, server!").unwrap();
            let rip_message: server::RipMessage = serde_yaml::from_str(&response).expect("Failed to parse YAML");
            assert_eq!(rip_message.version, 2);
        });

        let client_handle_two = thread::spawn(|| {
            let response = run_client("127.0.0.1:65434", "Hello, server!").unwrap();
            let rip_message: server::RipMessage = serde_yaml::from_str(&response).expect("Failed to parse YAML");
            assert_eq!(rip_message.version, 2);
        });

        // Attendre que les deux clients aient fini
        client_handle_one.join().unwrap();
        client_handle_two.join().unwrap();
    }
}
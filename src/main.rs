mod client;
mod server;

use std::thread;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <own address> <other address>", args[0]);
        return;
    }

    let own_address = args[1].clone(); // Cloner la chaîne pour éviter les problèmes de durée de vie
    let other_address = args[2].clone();

    // Lancer le serveur dans un thread séparé
    let server_handle = thread::spawn(move || {
        server::run_server(&own_address).expect("Server failed to run");
    });

    // Donner au serveur le temps de démarrer
    thread::sleep(std::time::Duration::from_millis(500));

    // Lancer un client pour envoyer un message
    let client_handle = thread::spawn(move || {
        match client::run_client(&other_address, "Hello, server!") {
            Ok(response) => println!("Response from server: {}", response),
            Err(e) => eprintln!("Client error: {}", e),
        }
    });

    // Attendre que les threads se terminent
    server_handle.join().unwrap();
    client_handle.join().unwrap();
}

mod client;
mod server;

use std::thread;
use std::env;
use std::time::Duration;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <own address> <other address>", args[0]);
        return;
    }

    let own_address = args[1].clone();
    let other_address = args[2].clone();
    let router = args[3].clone();

    // Lancer le serveur dans un thread séparé
    let server_handle = thread::spawn(move || {
        server::run_server(&own_address,&router).expect("Server failed to run");
    });

    // Attendre un peu que le serveur soit prêt
    thread::sleep(Duration::from_millis(500));

    // Lancer le client pour envoyer des mises à jour de routage périodiquement
    let client_handle = thread::spawn(move || {
        loop {
            if let Err(e) = client::send_routing_update(&other_address) {
                eprintln!("Failed to send routing update: {}", e);
            }
            thread::sleep(Duration::from_millis(500)); // Diffuser toutes les 500 ms
        }
    });

    server_handle.join().unwrap();
    client_handle.join().unwrap();
}

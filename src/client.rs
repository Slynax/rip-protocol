use std::io::{self, Read, Write};
use std::net::TcpStream;

pub fn run_client(address: &str, message: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(message.as_bytes())?;
    println!("Sent: {}", message);

    let mut response = String::new();
    let mut buffer = [0; 1024]; // Utilisez une taille de buffer raisonnable pour chaque lecture.
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break; // Fin de la connexion, si jamais le serveur ferme la connexion sans envoyer le marqueur.
        }
        response.push_str(&String::from_utf8_lossy(&buffer[..n]));
        if response.ends_with("#END#") {
            break; // Détecte le marqueur de fin et sort de la boucle.
        }
    }

    // Nettoyez la réponse en retirant le marqueur de fin.
    let clean_response = response.trim_end_matches("#END#").to_string();
    Ok(clean_response)
}

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::thread;

const ADDRESS: &str = "127.0.0.1:8080";
const BUFFER_SIZE:usize = 1024;
const MAX_NICKNAME_SIZE:usize = 32;
const SEND_FAIL_MESSAGE:&str = "Failed to send message";

fn main() {
    let listener = TcpListener::bind(ADDRESS).expect("Failed to bind address");
    let clients = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(stream, clients);
                });
            }

            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream,clients: Arc<Mutex<HashMap<String, TcpStream>>>) {
    let mut buffer = [0; BUFFER_SIZE];
    let nickname: String;

    match stream.read(&mut buffer) {
        Ok(0) => {
            println!("Connection closed before nickname provided");
            return;
        },
        Ok(size) => {
            nickname = String::from_utf8_lossy(&buffer[0..size]).to_string();
            if size > MAX_NICKNAME_SIZE{
                stream.write("Maximum nickname size is 32".as_bytes()).expect(SEND_FAIL_MESSAGE);
                return;
            }

            let mut clients_guard = clients.lock().expect("Failed to lock the mutex");

            if clients_guard.contains_key(&nickname) {
                stream.write(format!("{} already exists", nickname).as_bytes()).expect(SEND_FAIL_MESSAGE);
                return;
            }

            let welcome_message = format!("{} has joined!", nickname);
            for client in clients_guard.values_mut() {
                client.write_all(welcome_message.as_bytes()).expect(SEND_FAIL_MESSAGE);
            }
            clients_guard.insert(nickname.clone(), stream.try_clone().expect("Failed to clone the connection"));
        },
        Err(e) => {
            println!("Error reading from client {}", e);
            return;
        }
    }

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                let mut clients_guard = clients.lock().expect("Failed to lock mutex");
                let exit_message = format!("{} has left!", nickname);
                for client in clients_guard.values_mut() {
                    client.write_all(exit_message.as_bytes()).expect(SEND_FAIL_MESSAGE);
                }
                return;
            },
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[0..size]);

                let mut clients_guard = clients.lock().expect("Failed to lock mutex");
                for client in clients_guard.values_mut() {
                    client.write_all(message.as_bytes()).expect(SEND_FAIL_MESSAGE);
                }
            },
            Err(e) => {
                println!("Error reading from client {}", e);
                return;
            }
        }
    }
}

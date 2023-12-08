use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream};


pub struct Server {
    listener: TcpListener
}

impl Server {
    pub fn new() -> Server {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", 56565)).expect("Failed to bind to port 12345");
        println!("Server started!");
        Server {
            listener
        }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(client) => {
                    std::thread::spawn(|| {
                        handle_client(client);
                    });
                }
                Err(e) => eprintln!("Error, {}", e)
            }
        }
    }
}


fn handle_client(mut stream: TcpStream) {
    println!("Client connected: {:?}", stream.peer_addr());

    stream.write_all("Okay".as_bytes()).unwrap();

    // Example: Echo server, just echoes back the received data
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // Connection closed
                    println!("Client disconnected: {:?}", stream.peer_addr());
                    break;
                }

                println!("Received: {}", String::from_utf8_lossy(&buffer[0..bytes_read]));
                let response = handle_commands(String::from_utf8_lossy(&buffer[0..bytes_read]).to_string());
                match response {
                    Some(resp) => stream.write_all(resp.as_bytes()).unwrap(),
                    None => ()
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }
}

fn handle_commands(command: String) -> Option<String> {
    match command.as_str() {
        "devices" => Some(String::from("device1\ndevice2\n")),
        _ => None
    }
}
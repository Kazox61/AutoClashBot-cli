use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
        let device_streams = Arc::new(RwLock::new(HashMap::new()));
        let mut handles = vec![];
        for stream in self.listener.incoming() {
            let clone = Arc::clone(&device_streams);
            match stream {
                Ok(client) => {
                    let handle = std::thread::spawn(move || {
                        handle_client(client, &Arc::clone(&clone));
                    });
                    handles.push(handle);
                }
                Err(e) => eprintln!("Error, {}", e)
            }
        }
    }

    fn test(&self) {
        println!("Test");
    }
}


fn handle_client(mut stream: TcpStream, device_streams: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>) {
    println!("Client connected: {:?}", stream.peer_addr());

    stream.write_all("Okay".as_bytes()).unwrap();

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
                let response = handle_commands(String::from_utf8_lossy(&buffer[0..bytes_read]).to_string(), &device_streams.clone());
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

fn handle_commands(command: String, device_streams: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>) -> Option<String> {
    if command.starts_with("connect:") {
        let addr = command.split_once("connect:").unwrap().1;
        println!("Connect Command: {}", addr);
        return Some(handle_connect_command(addr.to_string(), &device_streams.clone()));
    }

    match command.as_str() {
        "devices" => {
            let mut response = String::from("Connected devices:\n");
            let devices = device_streams.read().unwrap();
            for device in devices.keys() {
                response += format!("\t{}\n", &device.to_string()).as_str();
            }
            return Some(response)
        },

        _ => None
    }
}

fn handle_connect_command(addr: String, device_streams: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>) -> String {
    let addr: SocketAddr = addr.parse().unwrap();
    match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)) {
        Ok(mut stream) => {
            let mut writer = device_streams.write().unwrap();
            writer.insert(addr, stream);
            return String::from("Connected to Socket");
        },
        Err(err) => match err.kind() {
            std::io::ErrorKind::TimedOut => {}
            _ => ()
        }
    }
    return String::from("");
}
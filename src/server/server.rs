use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream, SocketAddr, IpAddr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use byteorder::{BigEndian, WriteBytesExt};
use std::cmp::Ordering;
use crate::server::commands::{connect, devices};

#[repr(u16)]
pub enum DaemonCommands {
    StartInstance = 0,
    CloseInstance = 1,
    RestartInstance = 2,
    StopInstance = 3,
    ResumeInstance = 4,
}

pub struct DaemonCommand {
    command_id: u16,
    instance_id: Option<u16>,
    message: String
}

impl DaemonCommand {
    pub fn from(command: DaemonCommands, instance_id: Option<u16>, message: String) -> DaemonCommand {
        DaemonCommand {
            command_id: command as u16,
            instance_id,
            message
        }
    }
    fn new(command_id: u16, instance_id: Option<u16>, message: String) -> DaemonCommand {
        DaemonCommand { command_id, instance_id, message}
    }

    pub fn send(&self, mut stream: Arc<Mutex<TcpStream>>) {
        let msg_bytes = self.message.as_bytes();
        let version: u16 = 1;

        let mut data = vec![];
        let instance_id = match self.instance_id {
            Some(instance_id) => instance_id,
            None => 0
        };
        data.write_u16::<BigEndian>(instance_id).unwrap();
        data.write_u16::<BigEndian>(self.command_id).unwrap();
        data.write_u24::<BigEndian>(msg_bytes.len() as u32).unwrap();
        data.write_u16::<BigEndian>(version).unwrap();
        data.write_all(&msg_bytes).unwrap();

        stream.lock().unwrap().write_all(&data).unwrap()
    }
}


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
        let hashmap: HashMap<SocketAddr, Arc<Mutex<TcpStream>>> = HashMap::new();
        let device_streams = Arc::new(RwLock::new(hashmap));
        let mut handles = vec![];
        for stream in self.listener.incoming() {
            let clone = Arc::clone(&device_streams);
            match stream {
                Ok(client) => {

                    let handle = std::thread::spawn(move || {
                        handle_client(client, Arc::clone(&clone));
                    });
                    handles.push(handle);
                }
                Err(e) => eprintln!("Error, {}", e)
            }
        }
    }
}


fn handle_client(mut stream: TcpStream, device_streams: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>) {
    println!("Client connected: {:?}", stream.peer_addr());

    stream.write_all("Okay".as_bytes()).unwrap();

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Client disconnected: {:?}", stream.peer_addr());
                    break;
                }

                println!("Received: {}", String::from_utf8_lossy(&buffer[0..bytes_read]));
                let response = handle_commands(String::from_utf8_lossy(&buffer[0..bytes_read]).to_string(), device_streams.clone(), &stream);
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

fn handle_commands(command: String, device_streams: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>, client_stream: &TcpStream) -> Option<String> {
    let mut parts = command.split(":");
    let host = parts.next().expect("No Host provided");

    let device = match host {
        "host" => None,
        _ => {
            let port = parts.next().expect("No Port provided");
            Some(format!("{}:{}", host, port))
        }
    };

    let command = parts.next().expect("No Command provided");

    let arguments: Vec<&str> = parts.collect();

    return match command {
        "kill-server" => {
            for stream in device_streams.read().unwrap().values() {
                let stream = stream.lock().unwrap();
                stream.shutdown(std::net::Shutdown::Both).unwrap();
            }
            client_stream.shutdown(std::net::Shutdown::Both).unwrap();
            std::process::exit(0);
        },
        "connect" => Some(connect(device_streams, arguments)),
        "devices" => Some(devices(device_streams)),
        "start-instance" => {
            let instance = *arguments.get(0).unwrap();
            reqwest::blocking::get(format!("http:/localhost:8000/instance/{}/start", instance)).unwrap();
            Some("Success".to_string())
        },
        "close-instance" => {
            let instance = *arguments.get(0).unwrap();
            reqwest::blocking::get(format!("http:/localhost:8000/instance/{}/close", instance)).unwrap();
            Some("Success".to_string())
        },
        "restart-instance" => {
            let instance = *arguments.get(0).unwrap();
            reqwest::blocking::get(format!("http:/localhost:8000/instance/{}/restart", instance)).unwrap();
            Some("Success".to_string())
        },
        "stop-instance" => {
            let instance = *arguments.get(0).unwrap();
            reqwest::blocking::get(format!("http:/localhost:8000/instance/{}/stop", instance)).unwrap();
            Some("Success".to_string())
        },
        "resume-instance" => {
            let instance = *arguments.get(0).unwrap();
            reqwest::blocking::get(format!("http:/localhost:8000/instance/{}/resume", instance)).unwrap();
            Some("Success".to_string())
        }
        _ => None
    };
}
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
            let stream = get_stream_or_first(device, device_streams);
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => return Some(err)
            };
            let instance = *arguments.get(0).unwrap();
            let instance: u16 = instance.parse().unwrap();
            DaemonCommand::from(DaemonCommands::StartInstance, Some(instance), "".to_string()).send(stream);
            None
        },
        "close-instance" => {
            let stream = get_stream_or_first(device, device_streams);
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => return Some(err)
            };
            let instance = *arguments.get(0).unwrap();
            let instance: u16 = instance.parse().unwrap();
            DaemonCommand::from(DaemonCommands::CloseInstance, Some(instance), "".to_string()).send(stream);
            None
        },
        "restart-instance" => {
            let stream = get_stream_or_first(device, device_streams);
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => return Some(err)
            };
            let instance = *arguments.get(0).unwrap();
            let instance: u16 = instance.parse().unwrap();
            DaemonCommand::from(DaemonCommands::RestartInstance, Some(instance), "".to_string()).send(stream);
            None
        },
        "stop-instance" => {
            let stream = get_stream_or_first(device, device_streams);
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => return Some(err)
            };
            let instance = *arguments.get(0).unwrap();
            let instance: u16 = instance.parse().unwrap();
            DaemonCommand::from(DaemonCommands::StopInstance, Some(instance), "".to_string()).send(stream);
            None
        },
        "resume-instance" => {
            let stream = get_stream_or_first(device, device_streams);
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => return Some(err)
            };
            let instance = *arguments.get(0).unwrap();
            let instance: u16 = instance.parse().unwrap();
            DaemonCommand::from(DaemonCommands::ResumeInstance, Some(instance), "".to_string()).send(stream);
            None
        }
        _ => None
    };
}

fn get_stream_or_first(device: Option<String>, device_streams: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>) -> Result<Arc<Mutex<TcpStream>>, String> {
    return match device {
        None => {
            let devices = device_streams.read().unwrap();
            match devices.len().cmp(&1) {
                Ordering::Less => Err(String::from("No device connected")),
                Ordering::Equal => Ok(Arc::clone(devices.values().next().unwrap())),
                Ordering::Greater => Err(String::from("To many devices connected: Use -s to specify a device"))
            }
        }
        Some(device) => {
            let values = device.split_once(":").unwrap();
            let ip: IpAddr = match values.0.parse() {
                Err(_) => return Err(String::from("Invalid IP")),
                Ok(res) => res
            };

            let port:u16 = match values.1.parse() {
                Err(_) => return Err(String::from("Invalid Port")),
                Ok(res) => res
            };

            let addr = SocketAddr::new(ip, port);

            match device_streams.read().unwrap().get(&addr) {
                None => Err(String::from("No device with this serial")),
                Some(stream) => Ok(Arc::clone(&stream))
            }
        }
    }
}
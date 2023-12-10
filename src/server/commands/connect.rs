use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, RwLock};

pub fn connect(device_streams: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>, arguments: Vec<&str>) -> String {
    let host: IpAddr = match arguments.get(0) {
        None => return String::from("Invalid Host"),
        Some(&host) => match host.parse() {
            Err(_) => return String::from("Invalid Host"),
            Ok(host) => host
        }
    };
    let port = match arguments.get(1) {
        None => return String::from("Invalid Port"),
        Some(&port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => return String::from("Invalid Port")
        }
    };

    let addr = SocketAddr::new(host, port);
    match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)) {
        Ok(stream) => {
            let mut writer = device_streams.write().unwrap();
            writer.insert(addr, Arc::new(Mutex::new(stream)));
            return String::from("Connected to Socket");
        },
        Err(err) => match err.kind() {
            std::io::ErrorKind::TimedOut => {}
            _ => ()
        }
    }
    return String::from("");
}
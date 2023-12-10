use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, RwLock};

pub fn devices(device_streams: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>) -> String {
    let mut response = String::from("Connected devices:\n");
    let devices = device_streams.read().unwrap();
    for device in devices.keys() {
        response += format!("\t{}\n", &device.to_string()).as_str();
    }
    response
}
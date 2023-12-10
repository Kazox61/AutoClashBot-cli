use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use clap::builder::TypedValueParser;
use crate::server::server::{DaemonCommand, DaemonCommands};

pub fn start_instance(stream: Arc<Mutex<TcpStream>>, arguments: Vec<&str>) {
    let instance = *arguments.get(0).unwrap();
    let instance: u16 = instance.parse().unwrap();
    DaemonCommand::from(DaemonCommands::StartInstance, Some(instance), "".to_string()).send(stream);
}
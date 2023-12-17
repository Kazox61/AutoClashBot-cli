use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};

use clap::{Args, Parser, Subcommand};
use crate::cli::commands;
use crate::server::Server;
use crate::cli::commands::start_server;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "acb")]
#[command(about = "A CLI app for AutoClashBot(acb)", long_about = None)]
struct Cli {
    #[arg(short, long)]
    serial: Option<String>,
    #[command(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    StartServer,
    KillServer,
    ForkServer,
    #[command(arg_required_else_help = true)]
    Connect {
        daemon_server: String
    },
    Devices,
    #[command(arg_required_else_help = true)]
    StartInstance {
        instance_id: u16
    },
    #[command(arg_required_else_help = true)]
    CloseInstance {
        instance_id: u16
    },
    #[command(arg_required_else_help = true)]
    RestartInstance {
        instance_id: u16
    },
    #[command(arg_required_else_help = true)]
    StopInstance {
        instance_id: u16
    },
    #[command(arg_required_else_help = true)]
    ResumeInstance {
        instance_id: u16
    }
}

// HOST[:PORT]:COMMAND:ARG1:ARG2:...
pub fn send_cli_command(serial: Option<String>, command: String, options: Vec<String>, wait_for_response: bool) -> Option<String> {
    let serial = match serial {
        None => String::from("host"),
        Some(serial) => serial
    };

    let mut service = String::from(serial);
    service += format!(":{}", command).as_str();

    for option in options {
        service += format!(":{}", option).as_str();
    }

    let mut stream = connect_to_server();
    stream.write_all(service.as_bytes()).unwrap();

    if !wait_for_response {
        return None;
    }

    let mut buffer = [0; 1024];
    return match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            return Some(String::from_utf8_lossy(&buffer[0..bytes_read]).to_string());
        }
        Err(_) => None
    }

}

fn try_connect_to_server(addr: SocketAddr) -> Option<TcpStream> {
    match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)) {
        Ok(mut stream) => {
            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
                    if response == "Okay" {
                        return Some(stream);
                    }
                }
                Err(_) => ()
            }
        },
        Err(err) => match err.kind() {
            std::io::ErrorKind::TimedOut => (),
            _ => ()
        }
    }
    return None
}

fn connect_to_server() -> TcpStream {
    let addr: SocketAddr = "127.0.0.1:56565".parse().unwrap();

    let stream = try_connect_to_server(addr);

    return match stream {
        Some(stream) => stream,
        None => {
            start_server();
            std::thread::sleep(std::time::Duration::from_millis(100));
            let stream = try_connect_to_server(addr);
            println!("* server not running; starting now at localhost:56565");
            let stream = stream.expect("* server failed to start");
            println!("* server started successfully");
            stream
        }
    }
}

pub fn acb_cli() {
    let args = Cli::parse();

    let serial = args.serial;

    match args.command {
        Commands::StartServer => commands::start_server(),
        Commands::KillServer => commands::kill_server(),
        Commands::ForkServer => Server::new().run(), //@TODO: Maybe start this in a new thread
        Commands::Connect {daemon_server} => commands::connect(daemon_server),
        Commands::Devices => commands::devices(),
        Commands::StartInstance { instance_id} => commands::start_instance(serial, instance_id),
        Commands::CloseInstance { instance_id} => commands::close_instance(serial, instance_id),
        Commands::RestartInstance { instance_id} => commands::restart_instance(serial, instance_id),
        Commands::StopInstance { instance_id} => commands::stop_instance(serial, instance_id),
        Commands::ResumeInstance { instance_id} => commands::resume_instance(serial, instance_id),
        _ => ()
    }
}

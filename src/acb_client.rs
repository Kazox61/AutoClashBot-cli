use windows::{Win32::System::Threading::{CreateProcessW, DETACHED_PROCESS, STARTUPINFOW, PROCESS_INFORMATION}, core::{PWSTR, HSTRING}};
use std::{net::{TcpStream, SocketAddr}, io};
use std::io::{Write, Read};

pub fn acb_commandline(args: Vec<String>) {
    match args.get(1) {
        None => print_help_command(),
        Some(arg) => {
            if arg.starts_with("--") {

            }
            else if arg.starts_with("-") {
                
            }
            else{
                match arg.as_str() {
                    "help" => print_help_command(),
                    "devices" => get_devices(),
                    "start-server" => launch_server(),
                    "connect" => connect(args),
                    _ => {}
                }
            }
        }
    }
}

fn connect(args: Vec<String>) {
    match args.get(2) {
        None => println!("Please provide a second argument"),
        Some(arg) => {
            let addr: SocketAddr = match arg.parse() {
                Ok(addr) => addr,
                Err(err) => {
                    println!("The addr you provided is not vailid, {}", err);
                    return;
                }
            };

            match send_command(format!("connect:{}", addr.to_string())) {
                Some(msg) => println!("{}", msg),
                None => ()
            }            
        }
    }
}

fn get_devices() {
    let response = send_command("devices".to_string()).unwrap();
    println!("{}", response);
}

fn send_command(command: String) -> Option<String> {
    let mut stream = setup().unwrap();

    stream.write_all(command.as_bytes()).unwrap();

    let mut buffer = [0; 1024];
    return match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            return Some(String::from_utf8_lossy(&buffer[0..bytes_read]).to_string());
        }
        Err(_) => None
    }
}

fn setup() -> Option<TcpStream> {
    let addr: SocketAddr = "127.0.0.1:56565".parse().unwrap();
    match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)) {
        Ok(mut stream) => {
            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
                    if response == "Okay" {
                        return  Some(stream);
                    }
                }
                Err(_) => ()
            }
        },
        Err(err) => match err.kind() {
            io::ErrorKind::TimedOut => {
                println!("Timeout when trying to connect-server is not running yet, {}", err);
                launch_server();
                std::thread::sleep(std::time::Duration::from_millis(100));
                match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)) {
                    Ok(mut stream) => {
                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer) {
                            Ok(bytes_read) => {
                                let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
                                if response == "Okay" {
                                    return  Some(stream);
                                }
                            }
                            Err(_) => ()
                        }
                    },
                    Err(_) => ()
                }
                
            }
            _ => println!("Error connecting to the server, {}", err)
        }
    }
    return None;
}


fn print_help_command() {
    println!("Help");
}

fn launch_server() {
    let exe_path = &std::env::current_exe().unwrap().to_string_lossy().to_string();
    // string must be null terminated
    let mut args = String::from("fork-server\0").encode_utf16().collect::<Vec<u16>>();
    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    unsafe {
        let success = CreateProcessW(
            &HSTRING::from(exe_path),
            PWSTR(args.as_mut_ptr()),
            None,
            None,
            true,
            DETACHED_PROCESS,
            None,
            None,
            &mut si,
            &mut pi
        );
        match success {
            Err(err) => println!("Failed to fork the server as a detached process, {}", err),
            Ok(_) => println!("Forked the server as detached process with Id: {}. Use \"taskkill /F /PID {}\" to kill the process!", pi.dwProcessId, pi.dwProcessId)
        }
    }
}
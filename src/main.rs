mod acb_client;
mod acb;

use crate::acb_client::acb_commandline;
use crate::acb::Server;

const VERSION: &str = "0.0.0";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(0).cloned() {
        None => println!("Should not be possible!"),
        Some(arg) => {
            if arg.as_str() == "fork-server" {
                Server::new().run();
            }
        }
    }

    match args.get(1).cloned() {
        None => (),
        Some(arg) => {
            if arg.as_str() == "fork-server" {
                Server::new().run();
            }
        }
    }

    acb_commandline(args);
}

mod cli;
mod server;

use crate::cli::acb_cli;
pub const VERSION: &str = "0.0.0";

fn main() {
    acb_cli();
}

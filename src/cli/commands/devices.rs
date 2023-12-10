use crate::cli::commandline::send_cli_command;

pub fn devices() {
    let response = send_cli_command(
        None,
        String::from("devices"),
        vec![],
        true
    );
    println!("{}", response.unwrap())
}
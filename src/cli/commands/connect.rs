use crate::cli::commandline::send_cli_command;

pub fn connect(server: String) {
    let options = match server.split_once(":") {
        None => {
            println!("Invalid Serial");
            return;
        }
        Some(values) => vec![values.0.to_string(), values.1.to_string()]
    };
    let response = send_cli_command(
        None,
        String::from("connect"),
        options,
        true
    );

    println!("{}", response.unwrap())
}
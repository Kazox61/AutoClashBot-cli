use crate::cli::commandline::send_cli_command;

pub fn start_instance(serial: Option<String>, instance_id: u16) {
    let options = vec![instance_id.to_string()];
    let response = send_cli_command(
        serial,
        String::from("start-instance"),
        options,
        true
    );
    println!("{}", response.unwrap())
}
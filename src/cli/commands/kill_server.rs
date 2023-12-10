use crate::cli::commandline::send_cli_command;

pub fn kill_server() {
    send_cli_command(
        None,
        String::from("kill-server"),
        vec![],
        false
    );
}
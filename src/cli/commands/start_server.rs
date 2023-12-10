use windows::{Win32::System::Threading::{CreateProcessW, DETACHED_PROCESS, STARTUPINFOW, PROCESS_INFORMATION}, core::{PWSTR, HSTRING}};
pub fn start_server() {
    let exe_path = &std::env::current_exe().unwrap().to_string_lossy().to_string();
    // string must be null terminated
    // use the anything for first argument because when not using cmd the first argument is not the file name
    let mut args = String::from("AutoClashBot-cli.exe fork-server\0").encode_utf16().collect::<Vec<u16>>();
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
            Ok(_) => ()//println!("Forked the server as detached process with Id: {}. Use \"taskkill /F /PID {}\" to kill the process!", pi.dwProcessId, pi.dwProcessId)
        }
    }
}
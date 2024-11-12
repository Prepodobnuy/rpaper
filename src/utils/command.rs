use std::process::Command;


pub fn parse_command(command: &str, image_path: &str, original_image_path: &str, display: &str) -> String {
    return command
        .replace("{image}", image_path)
        .replace("{default_image}", original_image_path)
        .replace("{display}", display);
}

pub fn system(command: &str) {
    let mut child = Command::new("bash")
        .args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn command");

    let _exit_status = child.wait().expect("Failed to wait for command");
}

pub fn spawn(command: &str) {
    Command::new("bash")
        .args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn command");
}
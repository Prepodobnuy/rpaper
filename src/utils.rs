use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::error::Error;
use std::process::exit;

fn add_home_path_to_string(path: &str) -> String {
    let home_dir = match env::var_os("HOME") {
        Some(dir) => PathBuf::from(dir),
        _none => {
            eprintln!("Error: HOME environment variable is not set.");
            std::process::exit(1);
        }
    };

    return home_dir.join(path).into_os_string().into_string().unwrap();
}

pub fn parse_path(path: &str) -> String {
    if &path[0..1] == "~" {
        return add_home_path_to_string(&path[2..]);
    }
    return String::from(path);
}

pub fn parse_command(command: &str, image_path: &str, display: &str) -> String {
    //let res = command.to_owned() + " > /dev/null";
    return command
        .replace("{image}", image_path)
        .replace("{display}", display);
}

pub fn parse_args(default_config_path: String) -> (String, String, bool) {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        exit(1)
    }

    let mut image_path = String::from(&args[1]);
    let current_dir = std::env::current_dir().unwrap();
    image_path = current_dir.join(image_path).to_string_lossy().to_string();

    let mut cache_only: bool = false;

    let mut config_path = default_config_path;

    for (i, param) in args.iter().enumerate() {
        match param.as_str() {
            "-c" | "--conf" => config_path = parse_path(&args[i + 1]),
            "--cache-only" => cache_only = true,
            _ => {}
        }
    }

    return (config_path, image_path, cache_only);
}


pub fn spawn(command: String) {
    Command::new("bash")
        .args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Err");
}

pub fn start(command: &str) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("bash").args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let status = child.wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Command '{}' failed with status: {}", command, status).into())
    }
}
mod colorscheme;
mod daemon;
mod wallpaper;

use std::path::{PathBuf, Path};
use std::env;
use std::process::Command;

use sha2::{Sha256, Digest};

use crate::daemon::daemon::Daemon;

const SOCKET_PATH: &str = "/tmp/rpaper-daemon";
const CONFIG_PATH: &str = "~/.config/rpaper/config.json";
const CONFIG_DIR: &str = "~/.config/rpaper";
const CACHE_DIR: &str = "~/.cache/rpaper";
const COLORS_DIR: &str = "~/.cache/rpaper/rwal";
const COLORS_PATH: &str = "~/.cache/rpaper/rwal/colors";
const WALLPAPERS_DIR: &str = "~/.cache/rpaper/wallpapers";
//const DAEMON_NAME: &str = "rpaper-daemon";

pub fn expand_user(path: &str) -> String {
    if &path[0..1] != "~" {
        return String::from(path);
    }

    if let Some(home_dir) = env::var_os("HOME") {
        let stripped_path = path.strip_prefix("~/").unwrap_or(path);
        return PathBuf::from(home_dir).join(stripped_path).into_os_string().into_string().unwrap();
    }

    eprintln!("HOME environment variable is not set.");
    std::process::exit(1);
}

pub fn get_image_name(input: &str) -> String {
    let res;

    if let Some(pos) = input.rfind('/') {
        res = &input[pos + 1..]
    } else {
        res = input
    }

    String::from(res)
}

pub fn encode_string(string: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string);
    hex::encode(hasher.finalize())
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

fn main() {
    //system("killall rpaper-daemon");

    if Path::new(SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
    
    let mut daemon = Daemon::new();
    daemon.mainloop();
}

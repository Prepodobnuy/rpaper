mod colorscheme;
mod daemon;
mod logger;
mod template;
mod wallpaper;

use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nix::unistd::fork;
use sha2::{Digest, Sha256};

use crate::daemon::daemon::Daemon;

const SOCKET_PATH: &str = "/tmp/rpaper-daemon";
const CONFIG_PATH: &str = "~/.config/rpaper/config.json";
const CONFIG_DIR: &str = "~/.config/rpaper";
const CACHE_DIR: &str = "~/.cache/rpaper";
const COLORS_DIR: &str = "~/.cache/rpaper/rwal";
const COLORS_PATH: &str = "~/.cache/rpaper/rwal/colors";
const WALLPAPERS_DIR: &str = "~/.cache/rpaper/wallpapers";

//const DAEMON_NAME: &str = "rpaper-daemon";

pub fn unix_timestamp() -> u128 {
    let start = SystemTime::now();

    match start.duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => 0,
    }
}

pub fn expand_user(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    if &path[0..1] != "~" {
        return String::from(path);
    }

    if let Some(home_dir) = env::var_os("HOME") {
        let stripped_path = path.strip_prefix("~/").unwrap_or(path);
        return PathBuf::from(home_dir)
            .join(stripped_path)
            .into_os_string()
            .into_string()
            .unwrap();
    }

    panic!("HOME environment variable is not set.");
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
    let mut child = Command::new("nohup")
        .args(["bash", "-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("");
    let _exit_status = child.wait().expect("Failed to wait for command");
}

pub fn spawn(command: &str) {
    Command::new("nohup")
        .args(["bash", "-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("");
}

fn main() -> Result<(), ()> {
    if Path::new(SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(SOCKET_PATH);
        thread::sleep(Duration::from_millis(20));
    }

    let opts = Options::new();

    if opts.detach {
        match unsafe { fork().unwrap() } {
            nix::unistd::ForkResult::Parent { child: _ } => exit(0),
            nix::unistd::ForkResult::Child => {}
        }
    }

    let mut daemon = Daemon::new(opts.init_path);
    daemon.mainloop();
    Ok(())
}

struct Options {
    detach: bool,
    init_path: Option<String>,
}

impl Options {
    pub fn new() -> Self {
        let args = std::env::args().collect::<Vec<String>>();

        let detach = args.contains(&"-d".to_string());
        let mut init_path: Option<String> = None;

        if let Some(arg) = take_argument_after("-i", &args) {
            if std::path::Path::new(&expand_user(&arg)).exists() {
                init_path = Some(expand_user(&arg))
            }
        }

        Options { detach, init_path }
    }
}

fn take_argument_after(arg: &str, args: &Vec<String>) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if args[i - 1] == arg {
            return Some(a.to_string());
        }
    }
    None
}

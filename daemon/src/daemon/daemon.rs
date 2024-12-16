use std::sync::mpsc;
use std::thread;
use std::os::unix::net::UnixListener;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;
use std::fs::{self, File};
use std::path::Path;
use sha2::{Sha256, Digest};

use crate::SOCKET_PATH;
use crate::{daemon::config::Config, expand_user, CONFIG_PATH};
use crate::daemon::request::Request;

pub struct Daemon {
    config: Config,
    rx: mpsc::Receiver<MpscData>,
}

impl Daemon {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        // config init
        let mut config = Config::new();
        config.read(&expand_user(CONFIG_PATH));

        let dirs = vec![
            expand_user("~/.cache/rpaper"),
            expand_user("~/.cache/rpaper/wallpapers"),
            expand_user("~/.cache/rpaper/rwal"),
            expand_user("~/.config/rpaper"),
        ];

        // directory watcher
        start_directory_watcher(&dirs, tx.clone());
        // config watcher
        start_config_watcher(&expand_user(CONFIG_PATH), tx.clone());
        // socket_listener
        start_socket_listener(SOCKET_PATH, tx.clone());

        Daemon { config, rx }
    }

    pub fn mainloop(&mut self) {
        loop {
            if let Ok(received_data) = self.rx.try_recv() {
                match received_data {
                    MpscData::ConfigChanged => {
                        self.config.read(CONFIG_PATH);
                        println!("CONFIG CHANGED");
                    },
                    MpscData::ErrorCreatingDirectory => { println!("ERROR CREATING DIRECTORY"); },
                    MpscData::SuccesCreatingDirectory => { println!("DIRECTORY CREATED"); },
                    MpscData::ListenerRequest(message) => {
                        let mut request = Request::new(self.config.clone(), message);
                        request.process();
                        std::mem::drop(request);
                    },
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

pub enum MpscData {
    ConfigChanged,
    ErrorCreatingDirectory,
    SuccesCreatingDirectory,
    ListenerRequest(String),
}

fn start_socket_listener(socket_path: &str, tx: mpsc::Sender<MpscData>) {
    let listener;

    match UnixListener::bind(socket_path) {
        Ok(data) => {listener = data},
        Err(_) => {panic!("Unable to create socket")},
    }
    
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut reader = BufReader::new(stream);
                    let mut buffer = String::new();

                    reader.read_line(&mut buffer).unwrap();

                    let _ = tx.send(MpscData::ListenerRequest(buffer.trim().to_string()));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
    }});
}

fn start_directory_watcher(directories: &Vec<String>, tx: mpsc::Sender<MpscData>) {
    let directories = directories.clone();
    let _ = thread::Builder::new().name("directory watcher thread".to_string()).spawn(move || {
        loop {
            for dir in directories.iter() {
                let path = Path::new(&dir);
                if !path.exists() {
                    match fs::create_dir(path) {
                        Ok(_) => {
                            let _ = tx.send(MpscData::SuccesCreatingDirectory);
                        },
                        Err(_) => {
                            let _ = tx.send(MpscData::ErrorCreatingDirectory);
                        },
                    }
                }
            }            
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

fn start_config_watcher(config_path: &str, tx: mpsc::Sender<MpscData>) {
    let config_path = String::from(config_path);
    let _ = thread::Builder::new().name("config watcher thread".to_string()).spawn(move || {
        if let Ok(mut hash) = read_file(&config_path) {
            loop {
                if let Ok(new_hash) = read_file(&config_path) {
                    if new_hash != hash {
                        hash = new_hash;
                        let _ = tx.send(MpscData::ConfigChanged);
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
        panic!("CANNOT FIND CONFIG FILE AT {}", config_path)
    });
}

fn read_file(path: &str) -> Result<String, ()> {
    let mut hasher = Sha256::new();

    if let Ok(mut file) = File::open(path) {
        let mut buffer = Vec::new();
        if let Ok(_) = file.read_to_end(&mut buffer) {
            hasher.update(&buffer);
            let result = hasher.finalize();
            let hash_hex = hex::encode(result);
            return Ok(hash_hex);
        }
    }

    Err(())
}

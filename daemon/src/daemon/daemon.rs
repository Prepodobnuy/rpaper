use std::sync::mpsc;
use std::thread;
use std::os::unix::net::UnixListener;
use std::io::{self, BufRead, BufReader, Read};
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
                    MpscData::ConfigChanged(value) => {
                        self.config.read_from_string(value);
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
    ConfigChanged(String),
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
        if let Ok(file_caption) = read_file(&config_path) {
            let mut hash = file_caption.hash;
            loop {
                if let Ok(file_caption) = read_file(&config_path) {
                    if file_caption.hash != hash {
                        hash = file_caption.hash;
                        let _ = tx.send(MpscData::ConfigChanged(file_caption.caption));
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
        panic!("CANNOT FIND CONFIG FILE AT {}", config_path)
    });
}

struct FileCaption {
    hash: String,
    caption: String,
}

fn read_file(path: &str) -> Result<FileCaption, io::Error> {
    let mut hasher = Sha256::new();
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    let hash = hex::encode(hasher.finalize());
    let caption = String::from_utf8(buffer).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;

    Ok(FileCaption { hash, caption })
}

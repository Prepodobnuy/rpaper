use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::os::unix::net::UnixListener;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::time::Duration;
use std::fs::{self, File};
use std::path::Path;
use sha2::{Sha256, Digest};

use crate::{CACHE_DIR, COLORS_DIR, CONFIG_DIR, SOCKET_PATH, WALLPAPERS_DIR};
use crate::{daemon::config::Config, expand_user, CONFIG_PATH};
use crate::daemon::request::Request;

use super::config::JsonString;

pub struct Daemon {
    config: Config,
    rx: mpsc::Receiver<MpscData>,
    socket_tx: mpsc::Sender<MpscData>,
}

impl Daemon {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        // config init
        let mut config = Config::new();
        config.read(&expand_user(CONFIG_PATH));

        let dirs = vec![
            expand_user(CACHE_DIR),
            expand_user(WALLPAPERS_DIR),
            expand_user(COLORS_DIR),
            expand_user(CONFIG_DIR),
        ];

        // directory watcher
        start_directory_watcher(&dirs, tx.clone());
        // config watcher
        start_config_watcher(&expand_user(CONFIG_PATH), tx.clone());
        // socket_listener
        let socket_tx = start_socket_listener(SOCKET_PATH, tx.clone());

        Daemon { config, rx, socket_tx }
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
                    MpscData::DisplaysRequest => {
                        if let Some(displays) = &self.config.displays {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond(displays.json()));
                        }
                    },
                    MpscData::TemplatesRequest => {
                        if let Some(templates) = &self.config.templates {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond(templates.json()));
                        }
                    },
                    MpscData::ImageOpsRequest => {
                        if let Some(image_operations) = &self.config.image_operations {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond(image_operations.json()));
                        }
                    },
                    MpscData::RwalParamsRequest => {
                        if let Some(rwal_params) = &self.config.rwal_params {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond(rwal_params.json()));
                        }
                    },
                    MpscData::ConfigRequest => {
                        let _ = self.socket_tx.send(MpscData::ListenerRespond(self.config.json()));
                    },
                    _ => {},
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[derive(Clone)]
pub enum MpscData {
    ConfigChanged(String),
    ErrorCreatingDirectory,
    SuccesCreatingDirectory,
    ListenerRequest(String),
    ListenerRespond(String),
    DisplaysRequest,
    TemplatesRequest,
    ImageOpsRequest,
    RwalParamsRequest,
    ConfigRequest,
}

fn start_socket_listener(socket_path: &str, tx: mpsc::Sender<MpscData>) -> mpsc::Sender<MpscData> {
    let (listener_tx, listener_rx): (Sender<MpscData>, Receiver<MpscData>) = mpsc::channel();
    let listener = UnixListener::bind(socket_path).unwrap_or_else(|_| panic!("Unable to create socket"));

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = BufReader::new(&stream);
                    let mut buffer = String::new();

                    reader.read_line(&mut buffer).unwrap();
                    let request = buffer.trim().to_string();

                    if let Some(response) = handle_request(&request, &tx, &listener_rx) {
                        if let Err(e) = stream.write_all(format!("{}\n", response).as_bytes()) {
                            eprintln!("Failed to write to socket: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    });

    listener_tx
}

fn handle_request(request: &str, tx: &mpsc::Sender<MpscData>, listener_rx: &Receiver<MpscData>) -> Option<String> {
    let requests = [
        ("GET_DISPLAYS", MpscData::DisplaysRequest),
        ("GET_TEMPLATES", MpscData::TemplatesRequest),
        ("GET_IMAGE_OPS", MpscData::ImageOpsRequest),
        ("GET_RWAL_PARAMS", MpscData::RwalParamsRequest),
        ("GET_CONFIG", MpscData::ConfigRequest),
    ];

    for (pattern, data) in &requests {
        if request.contains(pattern) {
            if tx.send(data.clone()).is_ok() {
                if let Ok(value) = listener_rx.recv_timeout(Duration::from_millis(100)) {
                    if let MpscData::ListenerRespond(value) = value {
                        return Some(value);
                    }
                }
            }
        }
    }

    let _ = tx.send(MpscData::ListenerRequest(request.to_string()));

    None
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

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::os::unix::net::UnixListener;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::time::Duration;
use std::fs::{self, File};
use std::path::Path;
use sha2::{Sha256, Digest};

use crate::logger::logger::{err, info, log};
use crate::wallpaper::display::{get_cached_image_names, get_cached_image_paths};
use crate::{unix_timestamp, CACHE_DIR, COLORS_DIR, CONFIG_DIR, SOCKET_PATH, WALLPAPERS_DIR};
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
        let timestamp = unix_timestamp();
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

        info(&format!("Daemon initialized in {}ms.", unix_timestamp() - timestamp));
        Daemon { config, rx, socket_tx }
    }

    pub fn mainloop(&mut self) {
        info("Daemon started.");
        while Path::new(SOCKET_PATH).exists() {
            if let Ok(received_data) = self.rx.try_recv() {
                match received_data {
                    MpscData::ConfigChanged(value) => {
                        self.config.read_from_string(value);
                        info("Config changed.");
                    },
                    MpscData::ErrorCreatingDirectory => { err("Unable to create needed directory."); },
                    MpscData::SuccesCreatingDirectory => { info("Needed directory created."); },
                    MpscData::ListenerRequest(message) => {
                        log("Received wallpaper request.");
                        let mut request = Request::new(self.config.clone(), message);
                        request.process();
                        std::mem::drop(request);
                    },
                    MpscData::InfoRequest(val) => {
                        if let Some(respond) = process_info_request(self.config.clone(), val) {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond(respond));
                        }
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
    InfoRequest(InfoRequest),
}

#[derive(Clone)]
pub enum InfoRequest {
    DisplaysRequest,
    TemplatesRequest,
    ImageOpsRequest,
    RwalParamsRequest,
    ConfigRequest,
    CacheRequest(String),
    EmptyRequest,
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
                            err(&format!("Failed to write to socket: {}", e));
                        }
                    }
                }
                Err(e) => {
                    err(&format!("Error: {}", e));
                }
            }
        }
    });

    listener_tx
}

fn process_info_request (config: Config, request: InfoRequest) -> Option<String> {
    match request {
        InfoRequest::DisplaysRequest => {
            log("Displays request received.");
            if let Some(displays) = config.displays {
                Some(displays.json())
            } else {
                None
            }
        },
        InfoRequest::TemplatesRequest => {
            log("Templates request received.");
            if let Some(templates) = config.templates {
                Some(templates.json())
            } else {
                None
            }
        },
        InfoRequest::ImageOpsRequest => {
            log("Image operations request received.");
            if let Some(image_ops) = config.image_operations {
                Some(image_ops.json())
            } else {
                None
            }
        },
        InfoRequest::RwalParamsRequest => {
            log("Rwal params request received.");
            if let Some(rwal_params) = config.rwal_params {
                Some(rwal_params.json())
            } else {
                None
            }
        },
        InfoRequest::ConfigRequest => {
            log("Config request received.");
            Some(config.json())
        },
        InfoRequest::CacheRequest(val) => {
            log("Image cache info request received.");
            if let Some(img_ops) = config.image_operations {
                if let Some(displays) = config.displays {
                    let cached_image_paths = get_cached_image_paths(
                        &get_cached_image_names(&displays, &img_ops, &val),
                        WALLPAPERS_DIR,
                    );
                    let mut cached = true;
                    
                    for cache_path in &cached_image_paths {
                        if !Path::new(&expand_user(cache_path)).exists() {
                            cached = false;
                            break;
                        }
                    }

                    Some(
                        format!(
                            "{{\"paths\":[{}],\"cached\":{}}}",
                            cached_image_paths.into_iter().map(|path| {
                                format!("\"{}\"", path)
                            }).collect::<Vec<String>>().join(","),
                            cached.to_string(),
                        )
                    )

                }
                else {None}
            }
            else {None}
        },
        _ => {None}
    }
}

fn handle_request(request: &str, tx: &mpsc::Sender<MpscData>, listener_rx: &Receiver<MpscData>) -> Option<String> {
    let info_patterns = [
        "GET_DISPLAYS",
        "GET_TEMPLATES",
        "GET_IMAGE_OPS",
        "GET_RWAL_PARAMS",
        "GET_CONFIG",
        "GET_CACHE",
    ];

    for pat in info_patterns {
        if request.contains(pat) {
            let request = match pat {
                "GET_DISPLAYS"    => {MpscData::InfoRequest(InfoRequest::DisplaysRequest)},
                "GET_TEMPLATES"   => {MpscData::InfoRequest(InfoRequest::TemplatesRequest)},
                "GET_IMAGE_OPS"   => {MpscData::InfoRequest(InfoRequest::ImageOpsRequest)},
                "GET_RWAL_PARAMS" => {MpscData::InfoRequest(InfoRequest::RwalParamsRequest)},
                "GET_CONFIG"      => {MpscData::InfoRequest(InfoRequest::ConfigRequest)},
                "GET_CACHE"       => {
                    match request.contains("IMAGE") {
                        true => {
                            let parts: Vec<&str> = request.split_whitespace().collect();
                            let image_index = parts.iter().position(|&s| s == "IMAGE");

                            match image_index {
                                Some(index) => {
                                    if index + 1 < parts.len() {
                                        MpscData::InfoRequest(InfoRequest::CacheRequest(parts[index + 1].to_string()))
                                    } else {
                                        MpscData::InfoRequest(InfoRequest::EmptyRequest)
                                    }
                                },
                                None => {
                                    MpscData::InfoRequest(InfoRequest::EmptyRequest)
                                }
                            }
                        },
                        false => {MpscData::InfoRequest(InfoRequest::EmptyRequest)},
                    }
                },
                _ => {MpscData::InfoRequest(InfoRequest::EmptyRequest)},
            };
            if tx.send(request.clone()).is_ok() {
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
            thread::sleep(Duration::from_millis(100));
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
                thread::sleep(Duration::from_millis(100));
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

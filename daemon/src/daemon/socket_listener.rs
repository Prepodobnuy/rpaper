use std::os::unix::net::UnixListener;
use std::sync::mpsc::{self, Receiver, Sender};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;

use crate::logger::logger::{err, info};

use super::daemon::{InfoRequest, MpscData};

pub fn start_socket_listener(socket_path: &str, tx: mpsc::Sender<MpscData>) -> mpsc::Sender<MpscData> {
    info(&format!("Monitoring socket file at {}.", socket_path));
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


fn handle_request(request: &str, tx: &mpsc::Sender<MpscData>, listener_rx: &Receiver<MpscData>) -> Option<String> {
    let info_patterns = [
        "GET_DISPLAYS",
        "GET_TEMPLATES",
        "GET_SCHEME",
        "GET_IMAGE_OPS",
        "GET_RWAL_PARAMS",
        "GET_CONFIG",
        "GET_W_CACHE",
        "GET_C_CACHE",
    ];

    for pat in info_patterns {
        if request.contains(pat) {
            let request = match pat {
                "GET_DISPLAYS"    => {MpscData::InfoRequest(InfoRequest::DisplaysRequest)},
                "GET_TEMPLATES"   => {MpscData::InfoRequest(InfoRequest::TemplatesRequest)},
                "GET_SCHEME"      => {MpscData::InfoRequest(InfoRequest::CurrentColorSchemeRequest)}
                "GET_IMAGE_OPS"   => {MpscData::InfoRequest(InfoRequest::ImageOpsRequest)},
                "GET_RWAL_PARAMS" => {MpscData::InfoRequest(InfoRequest::RwalParamsRequest)},
                "GET_CONFIG"      => {MpscData::InfoRequest(InfoRequest::ConfigRequest)},
                "GET_W_CACHE"     => {
                    match request.contains("IMAGE") {
                        true => {
                            let parts: Vec<&str> = request.split_whitespace().collect();
                            let image_index = parts.iter().position(|&s| s == "IMAGE");

                            match image_index {
                                Some(index) => {
                                    if index + 1 < parts.len() {
                                        MpscData::InfoRequest(InfoRequest::WallpaperCacheRequest(parts[index + 1].to_string()))
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
                "GET_C_CACHE"     => {
                    match request.contains("IMAGE") {
                        true => {
                            let parts: Vec<&str> = request.split_whitespace().collect();
                            let image_index = parts.iter().position(|&s| s == "IMAGE");

                            match image_index {
                                Some(index) => {
                                    if index + 1 < parts.len() {
                                        MpscData::InfoRequest(InfoRequest::ColoschemeCacheRequest(parts[index + 1].to_string()))
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
                if let Ok(value) = listener_rx.recv_timeout(Duration::from_millis(10000)) {
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
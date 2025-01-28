use std::os::unix::net::UnixListener;
use std::sync::mpsc::{self, Receiver, Sender};
use std::io::{BufRead, BufReader, Write};
use std::thread;

use crate::logger::logger::{err, info};

use super::daemon::MpscData;
use super::request::handle_request;

pub fn start_socket_listener(socket_path: &str, sender: mpsc::Sender<MpscData>) -> mpsc::Sender<MpscData> {
    info(&format!("Monitoring socket file at {}.", socket_path));
    let (listener_sender, listener_receiver): (Sender<MpscData>, Receiver<MpscData>) = mpsc::channel();
    let listener = UnixListener::bind(socket_path).unwrap_or_else(|_| panic!("Unable to create socket"));

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = BufReader::new(&stream);
                    let mut buffer = String::new();

                    if let Err(_) = reader.read_line(&mut buffer) { continue; }

                    let request = buffer.trim().to_string();

                    if let Some(response) = handle_request(&request, &sender, &listener_receiver) {
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

    listener_sender
}
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::daemon::request::RequestHandler;
use crate::logger::logger::{err, info, log, warn};
use crate::{daemon::config::Config, expand_user, CONFIG_PATH};
use crate::{unix_timestamp, CACHE_DIR, COLORS_DIR, CONFIG_DIR, SOCKET_PATH, WALLPAPERS_DIR};

use super::config_watcher::start_config_watcher;
use super::directory_watcher::start_directory_watcher;
use super::socket_listener::start_socket_listener;

#[derive(Clone)]
pub enum MpscData {
    ConfigChanged(String),
    ErrorCreatingDirectory,
    SuccesCreatingDirectory,
    SocketRequest(String),
    Respond(RequestHandler),
}

pub struct Daemon {
    config: Config,
    receiver: mpsc::Receiver<MpscData>,
    socket_sender: mpsc::Sender<MpscData>,
}

impl Daemon {
    pub fn new(init_path: Option<String>) -> Self {
        let timestamp = unix_timestamp();
        let (sender, receiver) = mpsc::channel();

        let mut config = Config::new();
        config.read(&expand_user(CONFIG_PATH));

        start_directory_watcher(
            vec![
                expand_user(CACHE_DIR),
                expand_user(WALLPAPERS_DIR),
                expand_user(COLORS_DIR),
                expand_user(CONFIG_DIR),
            ],
            sender.clone(),
        );

        start_config_watcher(&expand_user(CONFIG_PATH), sender.clone());

        let socket_sender = start_socket_listener(SOCKET_PATH, sender.clone());

        info(&format!(
            "Daemon initialized in {}ms.",
            unix_timestamp() - timestamp
        ));

        if let Some(init_path) = init_path {
            log("processing init_path");
            if let Ok(message) = std::fs::read_to_string(init_path) {
                let mut request_handler = RequestHandler::new(config.clone(), message);
                request_handler.handle();
            } else {
                warn("init path does not exist");
            }
        }

        Daemon {
            config,
            receiver,
            socket_sender,
        }
    }

    pub fn mainloop(&mut self) {
        while Path::new(SOCKET_PATH).exists() {
            if let Ok(received_data) = self.receiver.try_recv() {
                match received_data {
                    MpscData::ConfigChanged(value) => {
                        self.config.read_from_string(value);
                        info("Config changed.");
                    }
                    MpscData::ErrorCreatingDirectory => {
                        err("Unable to create needed directory.");
                    }
                    MpscData::SuccesCreatingDirectory => {
                        info("Needed directory created.");
                    }
                    MpscData::SocketRequest(message) => {
                        log("Received socket request.");
                        let handler = RequestHandler::new(self.config.clone(), message);
                        let _ = self.socket_sender.send(MpscData::Respond(handler));
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::path::Path;

use crate::logger::logger::{err, info, log};
use crate::{unix_timestamp, CACHE_DIR, COLORS_DIR, CONFIG_DIR, SOCKET_PATH, WALLPAPERS_DIR};
use crate::{daemon::config::Config, expand_user, CONFIG_PATH};
use crate::daemon::request::Request;

use super::config_watcher::start_config_watcher;
use super::directory_watcher::start_directory_watcher;
use super::request::process_info_request;
use super::socket_listener::start_socket_listener;


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
    CurrentColorSchemeRequest,
    ImageOpsRequest,
    RwalParamsRequest,
    ConfigRequest,
    WallpaperCacheRequest(String),
    ColoschemeCacheRequest(String),
    EmptyRequest,
}

pub struct Daemon {
    config: Config,
    receiver: mpsc::Receiver<MpscData>,
    socket_sender: mpsc::Sender<MpscData>,
}

impl Daemon {
    pub fn new() -> Self {
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
            sender.clone()
        );

        start_config_watcher(&expand_user(CONFIG_PATH), sender.clone());

        let socket_sender = start_socket_listener(SOCKET_PATH, sender.clone());

        info(&format!("Daemon initialized in {}ms.", unix_timestamp() - timestamp));

        Daemon { config, receiver, socket_sender }
    }

    pub fn mainloop(&mut self) {
        while Path::new(SOCKET_PATH).exists() {
            if let Ok(received_data) = self.receiver.try_recv() {
                match received_data {
                    MpscData::ConfigChanged(value) => {
                        self.config.read_from_string(value);
                        info("Config changed.");
                    },
                    MpscData::ErrorCreatingDirectory => { 
                        err("Unable to create needed directory."); 
                    },
                    MpscData::SuccesCreatingDirectory => { 
                        info("Needed directory created."); 
                    },
                    MpscData::ListenerRequest(message) => {
                        log("Received wallpaper request.");
                        let mut request = Request::new(self.config.clone(), message);
                        request.process();
                        std::mem::drop(request);
                    },
                    MpscData::InfoRequest(request) => {
                        if let Some(respond) = process_info_request(self.config.clone(), request) {
                            let _ = self.socket_sender.send(MpscData::ListenerRespond(respond));
                        } else {
                            let _ = self.socket_sender.send(MpscData::ListenerRespond("{}".to_string()));
                        }
                    },
                    _ => {},
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
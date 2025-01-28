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
                        } else {
                            let _ = self.socket_tx.send(MpscData::ListenerRespond("".to_string()));
                        }
                    },
                    _ => {},
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
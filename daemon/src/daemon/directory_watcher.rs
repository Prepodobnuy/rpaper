use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::fs; 
use std::time::Duration;

use crate::daemon::daemon::MpscData;


pub fn start_directory_watcher(directories: Vec<String>, sender: mpsc::Sender<MpscData>) {
    let _ = thread::Builder::new().name("directory watcher thread".to_string()).spawn(move || {
        loop {
            for dir in directories.iter() {
                let path = Path::new(&dir);
                if !path.exists() {
                    match fs::create_dir(path) {
                        Ok(_) => {
                            let _ = sender.send(MpscData::SuccesCreatingDirectory);
                        },
                        Err(_) => {
                            let _ = sender.send(MpscData::ErrorCreatingDirectory);
                        },
                    }
                }
            }            
            thread::sleep(Duration::from_millis(100));
        }
    });
}
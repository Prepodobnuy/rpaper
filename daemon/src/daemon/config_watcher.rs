use std::fs::File;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::daemon::daemon::MpscData;

pub fn start_config_watcher(config_path: &str, sender: mpsc::Sender<MpscData>) {
    let config_path = String::from(config_path);
    let _ = thread::Builder::new()
        .name("config watcher thread".to_string())
        .spawn(move || {
            if let Ok(file_caption) = read_file(&config_path) {
                let mut hash = file_caption.hash;
                loop {
                    if let Ok(file_caption) = read_file(&config_path) {
                        if file_caption.hash != hash {
                            hash = file_caption.hash;
                            let _ = sender.send(MpscData::ConfigChanged(file_caption.caption));
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
    let caption =
        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(FileCaption { hash, caption })
}

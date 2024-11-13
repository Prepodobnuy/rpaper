use std::fs;
use std::io;
use std::env;
use std::path::PathBuf;


pub fn expand_user(path: &str) -> String {
    if &path[0..1] != "~" {
        return String::from(path);
    }

    if let Some(home_dir) = env::var_os("HOME") {
        let stripped_path = path.strip_prefix("~/").unwrap_or(path);
        return PathBuf::from(home_dir).join(stripped_path).into_os_string().into_string().unwrap();
    }

    eprintln!("Error: HOME environment variable is not set.");
    std::process::exit(1);
}

pub fn check_dirs() -> io::Result<()> {
    fs::create_dir_all(cache_dir())?;
    fs::create_dir_all(rwal_cache_dir())?;

    Ok(())
}

pub fn cache_dir() -> String {
    expand_user("~/.cache/rpaper/wallpapers")
}

pub fn rwal_cache_dir() -> String {
    expand_user("~/.cache/rpaper/rwal")
}
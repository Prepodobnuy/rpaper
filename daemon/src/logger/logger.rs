use std::time::SystemTime;
use chrono::{DateTime, Utc};

const RESET: &str = "\x1b[0m";

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";

fn get_datetime() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    format!(
        "{}[{}{}{}]{}",
        YELLOW,
        RESET,
        datetime.format("%H:%M:%S").to_string(),
        YELLOW,
        RESET,
    )
}

pub fn log(message: &str) {
    println!(
        "{}LOG{} {} {}",
        BLUE,
        RESET, 
        get_datetime(), 
        message
    );
}

pub fn info(message: &str) {
    println!(
        "{}INFO{} {} {}",
        MAGENTA,
        RESET, 
        get_datetime(), 
        message
    );
}

pub fn warn(message: &str) {
    println!(
        "{}WARN{} {} {}",
        YELLOW,
        RESET, 
        get_datetime(), 
        message
    );
}

pub fn err(message: &str) {
    println!(
        "{}ERR{} {} {}",
        RED,
        RESET, 
        get_datetime(), 
        message
    );
}
pub fn warn(value: &str) {
    println!("WARN: {}", value)
}

pub fn log(value: &str) {
    println!("LOG: {}", value)
}

pub fn error(value: &str) {
    println!("ERR: {}", value)
}


pub fn fatal(value: &str) {
    panic!("FATAL: {}", value)
}
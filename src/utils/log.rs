use std::fmt::Display;

fn log(message: String) {
    println!("{}", message)
}

pub fn info<T: Display>(msg: T) {
    log(format!("INFO: {}", msg)); 
}

pub fn success<T: Display>(msg: T) {
    log(format!("SUCCESS: {}", msg)); 
}

pub fn error<T: Display>(msg: T) {
    log(format!("ERROR: {}", msg)); 
}

pub fn warn<T: Display>(msg: T) {
    log(format!("WARN: {}", msg)); 
}

pub fn debug<T: Display>(msg: T) {
    log(format!("DEBUG: {}", msg)); 
}

pub fn pure<T: Display>(msg: T) {
    log(format!("{}", msg)); 
}

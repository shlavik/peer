use chrono::Local;
use std::fmt::Display;

pub fn log<T: Display>(string: T) {
    println!("{} | {}", Local::now().format("%H:%M:%S"), string);
}

pub fn log<T: std::fmt::Display>(string: T) {
    println!("{} | {}", chrono::Local::now().format("%H:%M:%S"), string);
}

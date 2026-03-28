use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
mod errors;
mod services;
mod state;

fn main() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    println!("Testing capture...");
    match services::capture::CaptureSession::start(0, PathBuf::from("test.mp4"), 30, stop_flag) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {:?}", e),
    }
}

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use wii_presenter::error::Error;
use wii_presenter::wiiuse_subsystem::start_wiiuse_subsystem;
use wiiuse_sys::wiiuse_init;

fn main() -> Result<(), Error> {
    println!("[main] start wii-presenter");
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let (join_handle, rc) = start_wiiuse_subsystem(shutdown_flag.clone());
    let res = join_handle.join().unwrap();
    Ok(())
}

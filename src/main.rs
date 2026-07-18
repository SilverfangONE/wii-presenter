use wii_presenter::wiiuse_subsystem::start_wiiuse_subsystem;
use wiiuse_sys::wiiuse_init;

fn main() {
    println!("start wii-presenter");
    let (jh, rc) = start_wiiuse_subsystem();
}

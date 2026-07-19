use std::{sync::Arc, thread::sleep, time::Duration};

use rs_wiiuse::{Wiimote, WiimoteButton, WiimoteId, Wiiuse};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

const SEARCH_TIMEOUT_SEC: u32 = 3;
const NORMAL_TIMEOUT_MS: u8 = 80;
const EXTENSION_TIMEOUT_MS: u8 = 100;

fn setup_controller(wiimote: &Wiimote) {
    wiimote.set_leds(rs_wiiuse::WiimoteLeds::new().on_1());
    wiimote.toggle_rumble();
    sleep(Duration::from_millis(300));
    wiimote.toggle_rumble();
}

fn run_presenter(wiiuse: Arc<Wiiuse>) -> Result<(), Error> {
    println!("[presenter] start presenter mode");
    loop {
        if let Some(wiimote) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
            if wiimote.is_disconnected() {
                println!("[presenter] disconnected wiimote");
                break;
            }

            if wiiuse.poll() < 1 {
                continue;
            }

            if wiimote.is_just_pressed(WiimoteButton::A) {
                println!("Button A wurde gedrückt!");
            }
            if wiimote.is_just_pressed(WiimoteButton::B) {
                println!("Button B wurde gedrückt!");
            }
        } else {
            println!("[presenter] no wiimotes anymore connected");
            break;
        }
    }
    println!("[presenter] exit presenter mode");
    Ok(())
}

fn start_presenter(wiiuse: Arc<Wiiuse>) -> Result<(), Error> {
    loop {
        // loop until connection establisehd to one wiimote
        println!("[Search] listing for incoming wiimote connections..");
        loop {
            if let Ok(1) = wiiuse.connect_all(SEARCH_TIMEOUT_SEC) {
                println!("[Search] connected to wiimote");
                break;
            }
        }

        // check if controller is present
        if let Some(wiimote) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
            setup_controller(&wiimote);

            // run presenter
            run_presenter(wiiuse.clone())?;
        }
    }
}

fn main() -> Result<(), Error> {
    println!("[main] start wii-presenter by @SilverfangONE");

    // setup wiiuse
    let wiiuse = Arc::new(Wiiuse::init(1));
    wiiuse.set_timeout(NORMAL_TIMEOUT_MS, EXTENSION_TIMEOUT_MS);

    start_presenter(wiiuse)
}

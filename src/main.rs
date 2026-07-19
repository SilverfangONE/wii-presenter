use std::{sync::Arc, thread::sleep, time::Duration};

use enigo::{Enigo, Keyboard, Mouse, Settings};
use rs_wiiuse::{Wiimote, WiimoteButton, WiimoteId, Wiiuse};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

const SEARCH_TIMEOUT_SEC: u32 = 6;
const NORMAL_TIMEOUT_MS: u8 = 80;
const EXTENSION_TIMEOUT_MS: u8 = 100;

fn rumble(wiimote: &Wiimote) {
    wiimote.toggle_rumble();
    sleep(Duration::from_millis(50));
    wiimote.toggle_rumble();
}

fn setup_controller(wiimote: &Wiimote) {
    wiimote.set_leds(rs_wiiuse::WiimoteLeds::new().on_1());
    wiimote.toggle_rumble();
    sleep(Duration::from_millis(300));
    wiimote.toggle_rumble();
}

fn run_presenter(wiiuse: Wiiuse) -> Result<(), Error> {
    println!("[presenter] start presenter mode");
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    loop {
        if let Some(wiimote) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
            if wiimote.is_disconnected() {
                println!("[presenter] disconnected wiimote");
                // drop
                break;
            }

            if wiiuse.poll() < 1 {
                continue;
            }

            if wiimote.is_just_pressed(WiimoteButton::A) {
                println!("Button A wurde gedrückt!");
                enigo
                    .key(enigo::Key::RightArrow, enigo::Direction::Press)
                    .unwrap();
                rumble(&wiimote);
            }
            if wiimote.is_just_pressed(WiimoteButton::B) {
                println!("Button B wurde gedrückt!");
                enigo
                    .key(enigo::Key::LeftArrow, enigo::Direction::Press)
                    .unwrap();
                rumble(&wiimote);
            }
            if wiimote.is_just_pressed(WiimoteButton::HOME) {
                println!("button Home ");
                break;
            }
        }
    }
    println!("[presenter] exit presenter mode");
    Ok(())
}

fn start_presenter() -> Result<(), Error> {
    loop {
        // setup wiiuse
        let wiiuse = Wiiuse::init(1);
        wiiuse.set_timeout(NORMAL_TIMEOUT_MS, EXTENSION_TIMEOUT_MS);

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
            run_presenter(wiiuse)?;
        }
        println!("restart presenter");
    }
}

fn main() -> Result<(), Error> {
    println!("start wii-presenter by @SilverfangONE");
    start_presenter()
}

use std::{
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use rs_wiiuse::{DEFAULT_EXPANSION_TIMEOUT, Wiimote, WiimoteButton, WiimoteId, Wiiuse};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

const SEARCH_TIMEOUT_SEC: u32 = 3;
const NORMAL_TIMEOUT_MS: u8 = 80;
const EXTENSTION_TIMEOUT_MS: u8 = 100;

fn setup_controller(wiimote: &Wiimote) {
    wiimote.set_leds(rs_wiiuse::WiimoteLeds::new().on_1());
    wiimote.toggle_rumble();
    sleep(Duration::from_millis(300));
    wiimote.toggle_rumble();
}

fn run_presenter(wiiuse: Arc<Wiiuse>) -> Result<(), Error> {
    loop {
        if wiiuse.poll() > 0
            && let Some(wm) = wiiuse.get_wiimote_by_id(WiimoteId(0))
        {
            if wm.is_just_pressed(WiimoteButton::)
            if wm.is_just_pressed(WiimoteButton::A) {
                println!("Button A wurde gedrückt!");
            }
            if wm.is_just_pressed(WiimoteButton::B) {
                println!("Button B wurde gedrückt!");
            }
        }
    }
}

fn start_presenter(wiiuse: Arc<Wiiuse>) -> Result<(), Error> {
    println!("[presenter] start connection listiner");

    loop {
        // loop until connection establisehd to one wiimote
        loop {
            if wiiuse.connect_all(SEARCH_TIMEOUT_SEC).unwrap() > 2 {
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

    start_presenter(wiiuse)
}

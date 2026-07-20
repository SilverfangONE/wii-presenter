use std::{thread::sleep, time::Duration};

use enigo::{Enigo, Keyboard, Mouse, Settings};
use rs_wiiuse::{Wiimote, WiimoteButton, WiimoteId, Wiiuse};
use tracing::Level;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

const AMT_WIIMOTES: u32 = 1;
const SEARCH_TIMEOUT_SEC: u32 = 6;
const NORMAL_TIMEOUT_MS: u8 = 80;
const EXTENSION_TIMEOUT_MS: u8 = 100;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    tracing::info!("start wii-presenter by @SilverfangONE");
    start_presenter()
}

fn start_presenter() -> Result<(), Error> {
    let span = tracing::span!(Level::INFO, "presenter");
    let _enter = span.enter();

    loop {
        let wiiuse = setup_wiiuse();
        connect_wiimote(&wiiuse);

        // check if controller is present
        if let Some(wiimote) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
            setup_controller(&wiimote);

            // run presenter
            run_presenter(wiiuse)?;
        }
        tracing::info!("restart wiimote search");
    }
}

fn setup_wiiuse() -> Wiiuse {
    let wiiuse = Wiiuse::init(AMT_WIIMOTES);
    wiiuse.set_timeout(NORMAL_TIMEOUT_MS, EXTENSION_TIMEOUT_MS);
    wiiuse
}

fn connect_wiimote(wiiuse: &Wiiuse) {
    let span = tracing::span!(Level::INFO, "connecting");
    let _enter = span.enter();
    tracing::info!("listing for incoming wiimote connections..");

    // loop until connection establisehd to one wiimote
    loop {
        // search for wiimote(s)
        loop {
            match wiiuse.find(SEARCH_TIMEOUT_SEC) {
                0 => continue,
                n => {
                    tracing::info!("found {} wiimote(s)", n);
                    break;
                }
            }
        }

        // connect to wiimote(s)
        match wiiuse.connect() {
            Ok(count) => {
                tracing::info!("connected to {} wiimote(s)", count);
                return;
            }
            _ => continue,
        };
    }
}

fn run_presenter(wiiuse: Wiiuse) -> Result<(), Error> {
    let span = tracing::span!(Level::INFO, "running");
    let _enter = span.enter();

    let wiimote_id = WiimoteId(0);
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    tracing::info!(
        "listing wiimote inputs (id = {} aka. light one active)..",
        wiimote_id.0
    );
    while let Some(wiimote) = wiiuse.get_wiimote_by_id(wiimote_id) {
        if wiimote.is_disconnected() {
            break;
        }

        if wiiuse.poll() < 1 {
            continue;
        }

        // disconnect current wiimote
        if wiimote.is_just_pressed(WiimoteButton::HOME) {
            wiimote.set_leds(rs_wiiuse::WiimoteLeds::new());
            break;
        }

        // switching between slides
        if wiimote.is_just_pressed(WiimoteButton::A) {
            enigo
                .key(enigo::Key::RightArrow, enigo::Direction::Click)
                .unwrap();
            rumble(&wiimote);
        }
        if wiimote.is_just_pressed(WiimoteButton::B) {
            enigo
                .key(enigo::Key::LeftArrow, enigo::Direction::Click)
                .unwrap();
            rumble(&wiimote);
        }

        // stepping forth in browser
        if wiimote.is_just_pressed(WiimoteButton::RIGHT) {
            enigo.key(enigo::Key::Alt, enigo::Direction::Press).unwrap();

            enigo
                .key(enigo::Key::RightArrow, enigo::Direction::Click)
                .unwrap();

            enigo
                .key(enigo::Key::Alt, enigo::Direction::Release)
                .unwrap();

            rumble(&wiimote);
        }

        // stepping behind in browser
        if wiimote.is_just_pressed(WiimoteButton::LEFT) {
            enigo.key(enigo::Key::Alt, enigo::Direction::Press).unwrap();

            enigo
                .key(enigo::Key::LeftArrow, enigo::Direction::Click)
                .unwrap();

            enigo
                .key(enigo::Key::Alt, enigo::Direction::Release)
                .unwrap();

            rumble(&wiimote);
        }

        // scrolling up in browser
        if wiimote.is_pressed(WiimoteButton::UP) {
            enigo.scroll(-6, enigo::Axis::Vertical).unwrap();
            rumble(&wiimote);
        }

        // scrolling down in browser
        if wiimote.is_pressed(WiimoteButton::DOWN) {
            enigo.scroll(6, enigo::Axis::Vertical).unwrap();
            rumble(&wiimote);
        }
    }
    tracing::info!(
        "wiimote (id = {}, aka. light one active) disconnected",
        wiimote_id.0
    );
    Ok(())
}

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

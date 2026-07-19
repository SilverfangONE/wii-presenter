use std::{thread::sleep, time::Duration};

use rs_wiiuse::{WiimoteButton, WiimoteId, Wiiuse};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

fn main() -> Result<(), Error> {
    println!("[main] start wii-presenter");
    let wiiuse = Wiiuse::init(1);
    wiiuse.connect_all(6).unwrap();
    if let Some(wiimote) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
        wiimote.set_leds(rs_wiiuse::WiimoteLeds::new().on_1());
        wiimote.toggle_rumble();
        sleep(Duration::from_millis(100));
        wiimote.toggle_rumble();
    }
    loop {
        wiiuse.poll();
        if let Some(wm) = wiiuse.get_wiimote_by_id(WiimoteId(0)) {
            if wm.is_button_pressed(WiimoteButton::B) {
                println!("Button A wurde gedrückt!");
            }
        }
    }
}

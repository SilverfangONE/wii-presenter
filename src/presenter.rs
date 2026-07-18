use crate::wiimote::Wiimote;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use gilrs::{Gilrs, Button, Event};

fn run_presenter<T: Wiimote>(wiimote_impl: T) {
    if !wiimote_impl.is_already_paired() {
        wiimote_impl.run_pairing();
    }

    println!("[Operations] Starte Presenter-Modus. Initialisiere Gilrs...");
    
    let mut gilrs = Gilrs::new().unwrap();
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    wiimote_impl.set_leds(led_mask);

    println!("[Operations] Höre auf Wiimote-Knöpfe. Bereit für Präsentation!");

    loop {
       while let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                gilrs::ev::EventType::ButtonPressed(Button::DPadRight, _) => {
                    println!("[Input] D-Pad Rechts -> Nächste Folie");
                    // .unwrap() fängt potenzielle Fehler beim Ausführen des Tastendrucks ab
                    enigo.key(Key::RightArrow, Direction::Click).unwrap();
                }
                gilrs::ev::EventType::ButtonPressed(Button::DPadLeft, _) => {
                    println!("[Input] D-Pad Links -> Vorherige Folie");
                    enigo.key(Key::LeftArrow, Direction::Click).unwrap();
                }
                _ => {
                    println!("{:?}", event);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10)); // CPU-Schonung beim Polling
    }
}
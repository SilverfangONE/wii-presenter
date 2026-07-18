use wiiuse_sys::wiiuse_version;

use crate::error::Error;
use core::slice;
use std::ffi::CStr;
use std::sync::Arc;
use std::{
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};

const AMT_WIIMOTES: i32 = 1;
const SEARCH_TIMEOUT_SEC: i32 = 5;

type WiimotePtrArr = *mut *mut wiiuse_sys::wiimote_t;
type WiimotePtr = *mut wiiuse_sys::wiimote_t;

#[allow(non_camel_case_types)]
pub enum WiimoteButton {
    WIIMOTE_BUTTON_ONE,
    WIIMOTE_BUTTON_TWO,
    WIIMOTE_BUTTON_B,
    WIIMOTE_BUTTON_A,
    WIIMOTE_BUTTON_MINUS,
    WIIMOTE_BUTTON_HOME,
    WIIMOTE_BUTTON_LEFT,
    WIIMOTE_BUTTON_RIGHT,
    WIIMOTE_BUTTON_DOWN,
    WIIMOTE_BUTTON_UP,
    WIIMOTE_BUTTON_PLUS,
}

#[allow(non_camel_case_types)]
pub enum WiiuseEvent {
    WIIUSE_NONE,
    WIIUSE_EVENT,
    WIIUSE_STATUS,
    WIIUSE_DISCONNECT,
    WIIUSE_READ_DATA,
}

impl From<u32> for WiiuseEvent {
    fn from(value: u32) -> Self {
        match value {
            wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_EVENT => WiiuseEvent::WIIUSE_EVENT,
            wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_STATUS => WiiuseEvent::WIIUSE_STATUS,
            wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_DISCONNECT => WiiuseEvent::WIIUSE_DISCONNECT,
            wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_READ_DATA => WiiuseEvent::WIIUSE_READ_DATA,
            wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_NONE | _ => WiiuseEvent::WIIUSE_NONE,
        }
    }
}

fn get_version() -> String {
    unsafe {
        let c_str_ptr = wiiuse_version();
        if c_str_ptr.is_null() {
            return "unknown".to_string();
        }
        let c_str = CStr::from_ptr(c_str_ptr);
        let str = c_str.to_str().map(|s| s.to_owned());
        match str {
            Ok(s) => s,
            Err(_) => "utf8-error".to_string(),
        }
    }
}

fn handle_wiiuse_event() {}

/// loops until at least 1 wiimote is connected
fn search_wiimotes(wm_ptr: WiimotePtrArr, amt_wiimotes: i32, timeout_sec: i32) -> i32 {
    loop {
        println!("[wiiuse] searching for pariring wiimotes..");
        let found = unsafe { wiiuse_sys::wiiuse_find(wm_ptr, amt_wiimotes, timeout_sec) };
        if found > 0 {
            println!("[wiiuse] found {} wiimotes", found);
            return found;
        }
    }
}

fn connect_wiimotes(wm_ptr: WiimotePtrArr, found_wm: i32) -> Option<i32> {
    let connected_wm = unsafe { wiiuse_sys::wiiuse_connect(wm_ptr, found_wm) };
    if connected_wm <= 0 {
        println!("[wiiuse] failed to connect to any wiimote");
        return None;
    }
    println!(
        "[wiiuse] Connected to {} wiimotes (of {} found).\n",
        connected_wm, found_wm
    );
    return Some(connected_wm);
}

fn run_wiiuse_subsystem(
    tx: Sender<WiiuseEvent>,
    shutdown_flag: Arc<AtomicBool>,
    amt_wm: i32,
) -> Result<(), Error> {
    println!("[wiiuse] use 'wiiuse' v{}", get_version());

    // init
    let wm_ptr_arr = unsafe { wiiuse_sys::wiiuse_init(amt_wm) };
    let wm_slices: &[WiimotePtr] = unsafe {
        slice::from_raw_parts(wm_ptr_arr as *const WiimotePtr, amt_wm.try_into().unwrap())
    };

    // find and connect wiimotes
    let found_wm = search_wiimotes(wm_ptr_arr, amt_wm, SEARCH_TIMEOUT_SEC);
    let _ = connect_wiimotes(wm_ptr_arr, found_wm).unwrap();

    // set proper led
    for i in 0..amt_wm as usize {
        unsafe {
            // 0x10 ist die erste LED.
            // Durch << i wird aus:
            // i=0 -> 0x10 (LED 1)
            // i=1 -> 0x20 (LED 2)
            // i=2 -> 0x40 (LED 3)
            // i=3 -> 0x80 (LED 4)
            let led_bitmask = 0x10 << i;

            // Konvertiere zu i32, da C-Enums/Makros in bindgen meist i32 sind
            wiiuse_sys::wiiuse_set_leds(wm_slices[i], led_bitmask as i32);
        }
    }

    // listen and poll events
    println!("[wiiuse] start communication subsystem");
    loop {
        if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let changed_wm = unsafe { wiiuse_sys::wiiuse_poll(wm_ptr_arr, amt_wm) };
        if changed_wm > 0 {
            for i in 0..amt_wm as usize {
                let ptr = wm_slices[i];
                if ptr.is_null() {
                    continue;
                }
                let wii_mote_ref = unsafe { &*ptr };
                let wiiuse_event: WiiuseEvent = wii_mote_ref.event.into();
                match wiiuse_event {
                    WiiuseEvent::WIIUSE_EVENT => {}
                    WiiuseEvent::WIIUSE_NONE | _ => {}
                }
            }
        }
    }

    // shutdown / disconnect all wiimotes
    unsafe {
        wiiuse_sys::wiiuse_cleanup(wm_ptr_arr, amt_wm);
    }
    Ok(())
}

pub fn start_wiiuse_subsystem(
    shutdown_flag: Arc<AtomicBool>,
) -> (JoinHandle<Result<(), Error>>, Receiver<WiiuseEvent>) {
    let (tx, rx) = channel();
    let jh = thread::spawn(move || run_wiiuse_subsystem(tx, shutdown_flag, AMT_WIIMOTES));
    return (jh, rx);
}

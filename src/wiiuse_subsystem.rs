use wiiuse_sys::wiiuse_version;

use crate::error::Error;
use core::slice;
use std::ffi::CStr;
use std::sync::Arc;
use std::{
    sync::{
        atomic::AtomicBool,
        mpsc::{self, Receiver, Sender, channel},
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
            
            _ => WiiuseEvent::WIIUSE_NONE,
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
    // init
    let wm_ptr_arr;
    unsafe {
        wm_ptr_arr = wiiuse_sys::wiiuse_init(amt_wm);
    }
    println!("[wiiuse] use 'wiiuse' v{}", get_version());

    // find and connect wiimotes
    let found_wm = search_wiimotes(wm_ptr_arr, amt_wm, SEARCH_TIMEOUT_SEC);
    let _connected_wm = connect_wiimotes(wm_ptr_arr, found_wm).unwrap();

    // listen and poll events
    println!("[wiiuse] start communication subsystem");
    let wm_slices: &[WiimotePtr] = unsafe {
        slice::from_raw_parts(wm_ptr_arr as *const WiimotePtr, amt_wm.try_into().unwrap())
    };

    loop {
        if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        unsafe {
            if wiiuse_sys::wiiuse_poll(wm_ptr_arr, amt_wm) > 0 {
                for i in 0..amt_wm as usize {
                    let ptr = wm_slices[i];
                    if ptr.is_null() {
                        continue;
                    }
                    let wii_mote_ref = unsafe { &*ptr};
                    match wii_mote_ref.event {
                        wiiuse_sys::WIIUSE_EVENT_TYPE_WIIUSE_CONNECT
                        _ => {}
                    }
                }
            }
        }


        for i in 0..amt_wm {
            
            // check for events
        while (1) { if (wiiuse_poll(wiimotes, 2)) { int i = 0; for (; i < 2; ++i) { switch (wiimotes[i]->event) { /* check the events here */ } } } }
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

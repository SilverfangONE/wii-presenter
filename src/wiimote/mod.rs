pub mod windows;
use crate::error::Error;

pub trait Wiimote {
    fn get_device_path(&self) -> Result<String, Error>; 
    fn set_leds(&self, led_mask: u8) -> Result<(), Error>;
    fn is_already_paired(&self) -> bool;
    fn run_pairing(&self) -> Result<(), Error>; 
}


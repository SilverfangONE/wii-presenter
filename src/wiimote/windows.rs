use std::ffi::c_void;
use std::mem::size_of;
use std::time::Duration;

use windows::Win32::Devices::Bluetooth::*;
use windows::Win32::Foundation::{BOOL, ERROR_SUCCESS, HANDLE};
use windows::core::GUID;
use windows::Win32::Devices::Bluetooth::*;
use windows::Win32::Devices::HumanInterfaceDevice::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::Foundation::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Devices::DeviceAndDriverInstallation::*;
use windows::Win32::Devices::HumanInterfaceDevice::*;
use windows::Win32::Foundation::{CloseHandle}; // CloseHandle hinzugefügt!
use windows::Win32::Storage::FileSystem::{FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL};
use windows::Win32::Devices::HumanInterfaceDevice::{HidD_SetOutputReport};
use std::mem::zeroed;
use windows::Win32::Storage::FileSystem::CreateFileW;

use crate::wiimote::Wiimote; 
use crate::error::Error;

struct WindowsWiimote {}

impl Wiimote for WindowsWiimote {
    fn get_device_path(&self) -> Result<String, Error> {
        unsafe {
            // 1. Liste aller HID-Schnittstellen abrufen
            let guid = GUID_DEVINTERFACE_HID;
            let device_info_set = SetupDiGetClassDevsW(
                Some(&guid),
                None,
                None,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            ).ok()?;

            let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
                cbSize: size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                ..zeroed()
            };

            // 2. Durch die Schnittstellen iterieren
            for i in 0..100 { // Wir suchen durch die ersten 100 Geräte
                if SetupDiEnumDeviceInterfaces(
                    device_info_set,
                    None,
                    &guid,
                    i,
                    &mut device_interface_data,
                ).is_err() {
                    break;
                }

                // 3. Detail-Info abrufen (Pfad)
                let mut required_size = 0;
                // Erster Aufruf um Größe zu bestimmen
                SetupDiGetDeviceInterfaceDetailW(
                    device_info_set,
                    &device_interface_data,
                    None,
                    0,
                    Some(&mut required_size),
                    None,
                );

                // Buffer für den Pfad erstellen
                let mut detail_data_buffer = vec![0u8; required_size as usize];
                let detail_data = detail_data_buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
                (*detail_data).cbSize = 5; // Für x64 Windows ist der Header 8 Byte, aber cbSize muss 5 sein wegen Alignment

                if SetupDiGetDeviceInterfaceDetailW(
                    device_info_set,
                    &device_interface_data,
                    Some(detail_data),
                    required_size,
                    None,
                    None,
                ).is_ok() {
                    // Den DevicePath aus der Struktur lesen
                    let path_ptr = (*detail_data).DevicePath.as_ptr();
                    let path = windows::core::PCWSTR(path_ptr).to_string().ok()?;
                    
                    // Wir suchen nach der Wiimote (Vendor ID 057e)
                    if path.to_uppercase().contains("VID_057E") {
                        println!("[SetupAPI] Wiimote gefunden: {}", path);
                        return Ok(path);
                    }
                }
            }
        }
        None
    }

    fn is_already_paired(&self) -> bool {
        false
    }

    fn run_pairing(&self) -> Result<(), Error> {
        Ok(())
    }

    fn set_leds(&self, led_mask: u8) -> Result<(), crate::error::Error> {
        unsafe {
            // 1. Öffne das HID-Device mit dem spezifischen Pfad, den dir Windows gibt
            let path_wide: Vec<u16> = OsStr::new(&self.get_device_path()?).encode_wide().chain(Some(0)).collect();
            let handle = CreateFileW(
                windows::core::PCWSTR(path_wide.as_ptr()),
                FILE_GENERIC_WRITE.0 as u32,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            ).unwrap();

            // 2. Das Wiimote-Protokoll: 
            // Report ID 0x11 = LED Steuerung
            // Das zweite Byte ist die Bitmaske für die LEDs (0x10 = LED 1 an)
            let report: [u8; 3] = [0x11, led_mask << 4, 0x00];

            // 3. Den Befehl senden
            HidD_SetOutputReport(handle, report.as_ptr() as *const _, report.len() as u32);
            
            CloseHandle(handle).unwrap();
        }
        Ok(())
    }


}

unsafe extern "system" fn auth_callback(
    _param: *const c_void,
    auth_event: *const BLUETOOTH_AUTHENTICATION_CALLBACK_PARAMS,
) -> BOOL {
    println!("[Callback] Juhu, Windows fragt unser Programm nach der PIN!");
    let event_data = unsafe { &*auth_event };

    if event_data.authenticationMethod != BLUETOOTH_AUTHENTICATION_METHOD_LEGACY {
        println!("[Callback] Fehler: Falsche Authentifizierungsmethode angefragt.");
        return BOOL(0);
    }

    // HIER IST DER WECHSEL: Wir nehmen die MAC-Adresse der Wiimote
    let wiimote_mac_u64 = unsafe { event_data.deviceInfo.Address.Anonymous.ullLong };
    let mac_bytes = wiimote_mac_u64.to_le_bytes();
    
    let mut pin_bytes = [0u8; 6];
    pin_bytes.copy_from_slice(&mac_bytes[0..6]);

    let mut auth_response = BLUETOOTH_AUTHENTICATE_RESPONSE {
        bthAddressRemote: event_data.deviceInfo.Address,
        authMethod: BLUETOOTH_AUTHENTICATION_METHOD_LEGACY,
        Anonymous: BLUETOOTH_AUTHENTICATE_RESPONSE_0 {
            pinInfo: BLUETOOTH_PIN_INFO {
                pin: [0; 16],
                pinLength: 6,
            },
        },
        negativeResponse: 0,
    };

    for (i, &byte) in pin_bytes.iter().enumerate() {
        unsafe {
            auth_response.Anonymous.pinInfo.pin[i] = byte;
        }
    }

    let result = unsafe { BluetoothSendAuthenticationResponseEx(None, &auth_response) };

    if result == ERROR_SUCCESS.0 {
        println!("[Callback] PIN (Sync-Button-Methode) erfolgreich an Windows übergeben!");
        BOOL(1)
    } else {
        println!("[Callback] Fehler beim Senden der PIN-Antwort an Windows.");
        BOOL(0)
    }
}

fn get_wiimote_device_path() -> Option<String> {
  
}

// Funktion, um die LEDs zu steuern
fn set_wiimote_leds(device_path: &str, led_mask: u8) {
   
}

fn is_wiimote_already_paired() -> bool {
    unsafe {
        let radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut h_radio = HANDLE::default();
        if BluetoothFindFirstRadio(&radio_params, &mut h_radio).is_err() { return false; }

        let search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: BOOL(1), // Wir suchen NUR nach bereits gekoppelten Geräten
            fReturnRemembered: BOOL(1),
            fReturnUnknown: BOOL(0),
            fReturnConnected: BOOL(0),
            fIssueInquiry: BOOL(0), // Kein neuer Scan, nur Cache prüfen
            cTimeoutMultiplier: 1,
            hRadio: h_radio,
        };

        let mut device_info = BLUETOOTH_DEVICE_INFO {
            dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };

        if let Ok(h_find) = BluetoothFindFirstDevice(&search_params, &mut device_info) {
            loop {
                let name_len = device_info.szName.iter().position(|&c| c == 0).unwrap_or(device_info.szName.len());
                let name = String::from_utf16_lossy(&device_info.szName[..name_len]);
                
                if name.contains("Nintendo") || name.contains("RVL-CNT") {
                    println!("[Setup] Wiimote ist bereits gekoppelt. Überspringe Pairing.");
                    return true; 
                }
                if BluetoothFindNextDevice(h_find, &mut device_info).is_err() { break; }
            }
            let _ = BluetoothFindDeviceClose(h_find);
        }
    }
    false
}

fn run_windows_pairing() -> bool {
    unsafe {
        // Lokale MAC holen
        let radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut h_radio: HANDLE = HANDLE::default();
        if BluetoothFindFirstRadio(&radio_params, &mut h_radio).is_err() {
            eprintln!("[Error] Kein Bluetooth-Adapter gefunden.");
            return false;
        }

        let mut radio_info = BLUETOOTH_RADIO_INFO {
            dwSize: size_of::<BLUETOOTH_RADIO_INFO>() as u32,
            ..Default::default()
        };
        BluetoothGetRadioInfo(h_radio, &mut radio_info);
        let host_mac = radio_info.address.Anonymous.ullLong;

        // Callback registrieren
        let host_mac_box = Box::new(host_mac);
        let param_ptr = Box::into_raw(host_mac_box) as *const c_void;
        let mut callback_handle = isize::default();
        
        BluetoothRegisterForAuthenticationEx(
            None,
            &mut callback_handle,
            Some(auth_callback),
            Some(param_ptr),
        );

        println!("[Pairing] Modus aktiv. Bitte Tasten 1+2 auf der Wiimote drücken...");

      // 1. Suchparameter (findet ALLES)
        let search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: BOOL(1), 
            fReturnRemembered: BOOL(1),
            fReturnUnknown: BOOL(1),
            fReturnConnected: BOOL(1),
            fIssueInquiry: BOOL(1),
            cTimeoutMultiplier: 2,
            hRadio: h_radio,
        };

        let mut wiimote_paired = false;

        // 2. DIESE SCHLEIFE IST DER SCHLÜSSEL
        while !wiimote_paired {
            println!("[Discovery] Scanne Umgebung nach Wiimote...");
            
            let mut device_info = BLUETOOTH_DEVICE_INFO {
                dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };

            // Scan starten
            if let Ok(h_find) = BluetoothFindFirstDevice(&search_params, &mut device_info) {
                loop {
                    let name_len = device_info.szName.iter().position(|&c| c == 0).unwrap_or(device_info.szName.len());
                    let name = String::from_utf16_lossy(&device_info.szName[..name_len]);

                    // Prüfen, ob das aktuell gefundene Gerät die Wiimote ist
                    if name.contains("Nintendo") || name.contains("RVL-CNT") {
                        println!("[Discovery] Wiimote gefunden! Sende Koppelungsanfrage...");

                        let auth_requirements = [
                            MITMProtectionNotRequired,
                            MITMProtectionNotRequiredBonding,
                            MITMProtectionNotRequiredGeneralBonding,
                        ];

                        let mut auth_succeeded = false;
                        for requirement in auth_requirements {
                            let auth_result = BluetoothAuthenticateDeviceEx(
                                None,
                                h_radio,
                                &mut device_info,
                                None,
                                requirement,
                            );

                            if auth_result == ERROR_SUCCESS.0 {
                                println!("[Discovery] Authentifizierung erfolgreich mit Anforderungen {:?}.", requirement);

                                let hid_guid = GUID::from_u128(0x00001124_0000_1000_8000_00805F9B34FB);
                                let _ = BluetoothSetServiceState(
                                    h_radio,
                                    &device_info,
                                    &hid_guid,
                                    1,
                                );

                                println!("[Discovery] HID-Service aktiviert. Controller ist einsatzbereit!");
                                wiimote_paired = true;
                                auth_succeeded = true;
                                break;
                            } else {
                                eprintln!(
                                    "[Discovery] Authentifizierung fehlgeschlagen für {:?}. Error-Code: {}",
                                    requirement,
                                    auth_result
                                );
                            }
                        }

                        if !auth_succeeded {
                            eprintln!("[Discovery] Keine Authentifizierungsstrategie war erfolgreich.");
                        }
                        break; // Bricht die innere loop ab, da wir unser Gerät gefunden haben
                    }

                    // Nächstes Gerät im aktuellen Scan prüfen
                    if BluetoothFindNextDevice(h_find, &mut device_info).is_err() {
                        break; // Alle Geräte in diesem Scan durch, innere loop abbrechen
                    }
                }
                let _ = BluetoothFindDeviceClose(h_find);
            }

            // Wenn die Wiimote nicht dabei war, kurz warten und noch mal scannen
            if !wiimote_paired {
                println!("[Discovery] Wiimote noch nicht gefunden. Versuche es in 2 Sekunden erneut...");
                std::thread::sleep(Duration::from_secs(2));
            }
        }

        let _ = BluetoothUnregisterAuthentication(callback_handle);
        let _ = Box::from_raw(param_ptr as *mut u64);
        println!("[Pairing] Phase beendet. Wechsel zu Operations-Modus.");
        true
    }
}


#![allow(clippy::upper_case_acronyms)]

use std::sync::OnceLock;

type HWND = *mut std::ffi::c_void;

// Modifier flags
const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
}

/// Parse modifier string like "ctrl+alt" or "alt+shift" into modifier flags
pub fn parse_modifiers(s: &str) -> u32 {
    let mut flags = 0u32;
    for part in s.to_lowercase().split('+') {
        match part.trim() {
            "alt" => flags |= MOD_ALT,
            "ctrl" | "control" => flags |= MOD_CONTROL,
            "shift" => flags |= MOD_SHIFT,
            "win" | "windows" | "meta" => flags |= MOD_WIN,
            _ => {}
        }
    }
    if flags == 0 {
        flags = MOD_CONTROL | MOD_ALT; // Default
    }
    flags | MOD_NOREPEAT
}

/// Convert a key name string to virtual key code
pub fn parse_vk(s: &str) -> u32 {
    let s = s.trim();
    // Single character: A-Z, 0-9
    if s.len() == 1 {
        let c = s.chars().next().unwrap();
        if c.is_ascii_uppercase() {
            return c as u32; // 'A'-'Z' → 0x41-0x5A
        }
        if c.is_ascii_lowercase() {
            return (c.to_ascii_uppercase()) as u32;
        }
        if c.is_ascii_digit() {
            return c as u32; // '0'-'9' → 0x30-0x39
        }
    }

    // Named keys
    match s.to_uppercase().as_str() {
        "F1" => 0x70,
        "F2" => 0x71,
        "F3" => 0x72,
        "F4" => 0x73,
        "F5" => 0x74,
        "F6" => 0x75,
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "F11" => 0x7A,
        "F12" => 0x7B,
        "SPACE" => 0x20,
        "TAB" => 0x09,
        "ENTER" | "RETURN" => 0x0D,
        "ESC" | "ESCAPE" => 0x1B,
        "BACKSPACE" | "BACK" => 0x08,
        "DELETE" | "DEL" => 0x2E,
        "INSERT" | "INS" => 0x2D,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" | "PGUP" => 0x21,
        "PAGEDOWN" | "PGDN" => 0x22,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        "NUMPAD0" => 0x60,
        "NUMPAD1" => 0x61,
        "NUMPAD2" => 0x62,
        "NUMPAD3" => 0x63,
        "NUMPAD4" => 0x64,
        "NUMPAD5" => 0x65,
        "NUMPAD6" => 0x66,
        "NUMPAD7" => 0x67,
        "NUMPAD8" => 0x68,
        "NUMPAD9" => 0x69,
        _ => {
            // Default to 'T' for Tip Clock
            'T' as u32
        }
    }
}

pub struct HotKey {
    modifiers: u32,
    vk: u32,
    registered: bool,
}

static HOTKEY_INSTANCE: OnceLock<std::sync::Mutex<HotKey>> = OnceLock::new();

impl HotKey {
    pub fn init(mod_str: &str, key_str: &str) {
        let instance = HotKey {
            modifiers: parse_modifiers(mod_str),
            vk: parse_vk(key_str),
            registered: false,
        };
        HOTKEY_INSTANCE.set(std::sync::Mutex::new(instance)).ok();
    }

    pub fn register(hwnd: HWND) {
        if let Some(lock) = HOTKEY_INSTANCE.get() {
            let mut hk = lock.lock().unwrap();
            if !hk.registered {
                unsafe {
                    // Use ID 1 for the hotkey
                    let result = RegisterHotKey(hwnd, 1, hk.modifiers, hk.vk);
                    if result != 0 {
                        hk.registered = true;
                    }
                }
            }
        }
    }

    pub fn unregister(hwnd: HWND) {
        if let Some(lock) = HOTKEY_INSTANCE.get() {
            let mut hk = lock.lock().unwrap();
            if hk.registered {
                unsafe {
                    UnregisterHotKey(hwnd, 1);
                }
                hk.registered = false;
            }
        }
    }

    /// Re-register with new modifiers/key (after config reload)
    pub fn update(hwnd: HWND, mod_str: &str, key_str: &str) {
        if let Some(lock) = HOTKEY_INSTANCE.get() {
            let mut hk = lock.lock().unwrap();
            let new_mods = parse_modifiers(mod_str);
            let new_vk = parse_vk(key_str);
            if hk.modifiers != new_mods || hk.vk != new_vk || !hk.registered {
                // Unregister old
                if hk.registered {
                    unsafe {
                        UnregisterHotKey(hwnd, 1);
                    }
                }
                hk.modifiers = new_mods;
                hk.vk = new_vk;
                // Register new
                unsafe {
                    let result = RegisterHotKey(hwnd, 1, hk.modifiers, hk.vk);
                    hk.registered = result != 0;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_registered() -> bool {
        HOTKEY_INSTANCE
            .get()
            .map(|lock| lock.lock().unwrap().registered)
            .unwrap_or(false)
    }
}

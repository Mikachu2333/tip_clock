#![allow(clippy::upper_case_acronyms)]

use std::sync::atomic::{AtomicIsize, Ordering};

type HWND = *mut std::ffi::c_void;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;
const HOTKEY_ID: i32 = 1;

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
}

static REGISTERED_HWND: AtomicIsize = AtomicIsize::new(0);

/// Parse a modifier string such as `ctrl+alt`.
///
/// Empty input means no modifier. Unknown names are rejected instead of
/// silently changing the requested shortcut.
pub fn parse_modifiers(s: &str) -> Result<u32, String> {
    let mut flags = 0u32;
    for part in s.split('+').map(str::trim).filter(|part| !part.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "alt" => flags |= MOD_ALT,
            "ctrl" | "control" => flags |= MOD_CONTROL,
            "shift" => flags |= MOD_SHIFT,
            "win" | "windows" | "meta" => flags |= MOD_WIN,
            other => return Err(format!("unknown hotkey modifier: '{other}'")),
        }
    }
    Ok(flags)
}

/// Convert a key name to a Win32 virtual-key code.
pub fn parse_vk(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if s.len() == 1 {
        let c = s.as_bytes()[0];
        if c.is_ascii_alphabetic() || c.is_ascii_digit() {
            return Ok(c.to_ascii_uppercase() as u32);
        }
    }
    match s.to_ascii_uppercase().as_str() {
        "F1" => Ok(0x70),
        "F2" => Ok(0x71),
        "F3" => Ok(0x72),
        "F4" => Ok(0x73),
        "F5" => Ok(0x74),
        "F6" => Ok(0x75),
        "F7" => Ok(0x76),
        "F8" => Ok(0x77),
        "F9" => Ok(0x78),
        "F10" => Ok(0x79),
        "F11" => Ok(0x7A),
        "F12" => Ok(0x7B),
        "SPACE" => Ok(0x20),
        "TAB" => Ok(0x09),
        "ENTER" | "RETURN" => Ok(0x0D),
        "ESC" | "ESCAPE" => Ok(0x1B),
        "BACKSPACE" | "BACK" => Ok(0x08),
        "DELETE" | "DEL" => Ok(0x2E),
        "HOME" => Ok(0x24),
        "END" => Ok(0x23),
        "PAGEUP" | "PGUP" => Ok(0x21),
        "PAGEDOWN" | "PGDN" => Ok(0x22),
        "UP" => Ok(0x26),
        "DOWN" => Ok(0x28),
        "LEFT" => Ok(0x25),
        "RIGHT" => Ok(0x27),
        _ => Err(format!("unknown key: '{s}'")),
    }
}

/// Register a non-repeating, OS-managed global hotkey for `target_hwnd`.
pub fn init(mod_str: &str, key_str: &str, target_hwnd: HWND) -> Result<(), String> {
    if target_hwnd.is_null() {
        return Err("cannot register hotkey for a null window".into());
    }
    let modifiers = parse_modifiers(mod_str)? | MOD_NOREPEAT;
    let vk = parse_vk(key_str).map_err(|e| format!("invalid hotkey key: {e}"))?;

    // SAFETY: target_hwnd is a live window created by gui.rs; the ID is private
    // to that window, and modifiers/vk are validated integer flags.
    if unsafe { RegisterHotKey(target_hwnd, HOTKEY_ID, modifiers, vk) } == 0 {
        let error = std::io::Error::last_os_error();
        let error_text = format!(
            "RegisterHotKey failed (Win32 error {}): {error}",
            error.raw_os_error().unwrap_or_default()
        );
        win_msgbox_timeout::error_msgbox("Error", &error_text, 3);
        return Err(error_text);
    }
    REGISTERED_HWND.store(target_hwnd as isize, Ordering::Release);
    Ok(())
}

pub fn shutdown() {
    let hwnd = REGISTERED_HWND.swap(0, Ordering::AcqRel) as HWND;
    if !hwnd.is_null() {
        // SAFETY: this exact HWND/ID pair was registered successfully in init.
        unsafe { UnregisterHotKey(hwnd, HOTKEY_ID) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keys() {
        assert_eq!(parse_vk("b").unwrap(), 'B' as u32);
        assert_eq!(parse_vk("F12").unwrap(), 0x7B);
        assert_eq!(parse_vk("enter").unwrap(), 0x0D);
        assert!(parse_vk("F13").is_err());
    }

    #[test]
    fn parse_modifier_combinations() {
        assert_eq!(parse_modifiers("ctrl+alt").unwrap(), MOD_CONTROL | MOD_ALT);
        assert_eq!(parse_modifiers("Win+Shift").unwrap(), MOD_WIN | MOD_SHIFT);
        assert_eq!(parse_modifiers("").unwrap(), 0);
        assert!(parse_modifiers("ctrl+hyper").is_err());
    }
}

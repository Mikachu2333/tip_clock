#![allow(clippy::upper_case_acronyms)]

use std::sync::atomic::{AtomicIsize, Ordering};

type HWND = *mut std::ffi::c_void;
type HHOOK = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;

const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYDOWN: u32 = 0x0100;
const WM_SYSKEYDOWN: u32 = 0x0104;

/// Custom message posted to the main window when hotkey fires.
pub const WM_USER_HOTKEY: u32 = 0x0401;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowsHookExW(
        idHook: i32,
        lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
        hMod: HINSTANCE,
        dwThreadId: u32,
    ) -> HHOOK;

    fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn GetKeyState(nVirtKey: i32) -> i16;
    fn PostMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

// ── Hotkey state ──────────────────────────────

static TARGET_HWND: AtomicIsize = AtomicIsize::new(0);
static HOTKEY_VK: AtomicIsize = AtomicIsize::new(0);
static HOTKEY_MODS: AtomicIsize = AtomicIsize::new(0);
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

fn debug_log(s: impl ToString) {
    crate::audio::debug_log(s);
}

/// Parse modifier string like "ctrl+alt" into modifier flags.
pub fn parse_modifiers(s: &str) -> u32 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
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
        flags = MOD_CONTROL | MOD_ALT;
    }
    flags
}

/// Convert a key name string to virtual key code.
pub fn parse_vk(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if s.len() == 1 {
        let c = s.chars().next().unwrap();
        if c.is_ascii_uppercase() {
            return Ok(c as u32);
        }
        if c.is_ascii_lowercase() {
            return Ok(c.to_ascii_uppercase() as u32);
        }
        if c.is_ascii_digit() {
            return Ok(c as u32);
        }
    }
    match s.to_uppercase().as_str() {
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

// ── Low-level keyboard hook callback ──────────

unsafe extern "system" fn keyboard_hook_callback(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    if n_code < 0 {
        return unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) };
    }

    if w_param == WM_KEYDOWN as usize || w_param == WM_SYSKEYDOWN as usize {
        let vk_code = unsafe { *(l_param as *const u32) } as i32;
        let expected_vk = HOTKEY_VK.load(Ordering::Relaxed) as i32;
        let expected_mods = HOTKEY_MODS.load(Ordering::Relaxed) as u32;

        if vk_code == expected_vk {
            // Check modifiers
            let mut current_mods = 0u32;
            if (unsafe {
                GetKeyState(0x10 /* VK_SHIFT */)
            } as u32
                & 0x8000)
                != 0
            {
                current_mods |= MOD_SHIFT;
            }
            if (unsafe {
                GetKeyState(0x11 /* VK_CONTROL */)
            } as u32
                & 0x8000)
                != 0
            {
                current_mods |= MOD_CONTROL;
            }
            // Alt: check both left and right Alt, and the LLKHF_ALTDOWN flag
            let alt_down = (unsafe {
                GetKeyState(0x12 /* VK_MENU */)
            } as u32
                & 0x8000)
                != 0;
            if alt_down {
                current_mods |= MOD_ALT;
            }
            // Win key: check both left and right Windows keys
            let lwin = (unsafe {
                GetKeyState(0x5B /* VK_LWIN */)
            } as u32
                & 0x8000)
                != 0;
            let rwin = (unsafe {
                GetKeyState(0x5C /* VK_RWIN */)
            } as u32
                & 0x8000)
                != 0;
            if lwin || rwin {
                current_mods |= MOD_WIN;
            }

            if current_mods == expected_mods {
                let target = TARGET_HWND.load(Ordering::Relaxed) as HWND;
                if !target.is_null() {
                    debug_log("[hotkey] keyboard hook matched, posting WM_USER_HOTKEY\n");
                    let posted = unsafe { PostMessageW(target, WM_USER_HOTKEY, 0, 0) };
                    if posted == 0 {
                        debug_log("[hotkey] PostMessageW failed\n");
                    }
                }
                // Don't block the key — let other apps see it too
            }
        }
    }

    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

// ── Public API ────────────────────────────────

/// Install the low-level keyboard hook and configure the hotkey.
pub fn init(mod_str: &str, key_str: &str, target_hwnd: HWND) -> Result<(), String> {
    TARGET_HWND.store(target_hwnd as isize, Ordering::Relaxed);

    let mods = parse_modifiers(mod_str);
    let vk = parse_vk(key_str).map_err(|e| format!("invalid hotkey key: {e}"))?;

    debug_log(format!(
        "[hotkey] installing keyboard hook: mod=0x{mods:x}, vk=0x{vk:x}\n"
    ));

    HOTKEY_VK.store(vk as isize, Ordering::Relaxed);
    HOTKEY_MODS.store(mods as isize, Ordering::Relaxed);

    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            keyboard_hook_callback,
            hinst,
            0, // global hook (0 = all threads)
        )
    };

    if hook.is_null() {
        return Err("SetWindowsHookExW failed".into());
    }

    HOOK_HANDLE.store(hook as isize, Ordering::Relaxed);

    debug_log("[hotkey] keyboard hook installed successfully\n");

    Ok(())
}

// ───────────────────────────────────────────────
//  Unit tests
// ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vk_single_char() {
        assert_eq!(parse_vk("A").unwrap(), 'A' as u32);
        assert_eq!(parse_vk("b").unwrap(), 'B' as u32);
        assert_eq!(parse_vk("5").unwrap(), '5' as u32);
    }

    #[test]
    fn test_parse_vk_function_keys() {
        assert_eq!(parse_vk("F1").unwrap(), 0x70);
        assert_eq!(parse_vk("f12").unwrap(), 0x7B);
        assert_eq!(parse_vk("F10").unwrap(), 0x79);
    }

    #[test]
    fn test_parse_vk_special_keys() {
        assert_eq!(parse_vk("SPACE").unwrap(), 0x20);
        assert_eq!(parse_vk("enter").unwrap(), 0x0D);
        assert_eq!(parse_vk("esc").unwrap(), 0x1B);
        assert_eq!(parse_vk("DELETE").unwrap(), 0x2E);
        assert_eq!(parse_vk("home").unwrap(), 0x24);
        assert_eq!(parse_vk("UP").unwrap(), 0x26);
    }

    #[test]
    fn test_parse_vk_unknown() {
        assert!(parse_vk("F13").is_err());
        assert!(parse_vk("abc").is_err());
        assert!(parse_vk("unknown").is_err());
    }

    #[test]
    fn test_parse_vk_whitespace() {
        assert_eq!(parse_vk("  A  ").unwrap(), 'A' as u32);
        assert_eq!(parse_vk("  F5  ").unwrap(), 0x74);
    }

    #[test]
    fn test_parse_modifiers() {
        assert_eq!(parse_modifiers("ctrl+alt"), MOD_CONTROL | MOD_ALT);
        assert_eq!(parse_modifiers("Win+Alt"), MOD_WIN | MOD_ALT);
        assert_eq!(parse_modifiers("shift"), MOD_SHIFT);
        assert_eq!(parse_modifiers(""), 0);
    }

    #[test]
    fn test_parse_modifiers_case_insensitive() {
        assert_eq!(parse_modifiers("CTRL+ALT"), MOD_CONTROL | MOD_ALT);
        assert_eq!(parse_modifiers("Win+Shift"), MOD_WIN | MOD_SHIFT);
    }

    #[test]
    fn test_parse_modifiers_default() {
        // Unknown modifiers default to Ctrl+Alt
        assert_eq!(parse_modifiers("xyz"), MOD_CONTROL | MOD_ALT);
    }
}

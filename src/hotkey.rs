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
pub fn parse_vk(s: &str) -> u32 {
    let s = s.trim();
    if s.len() == 1 {
        let c = s.chars().next().unwrap();
        if c.is_ascii_uppercase() {
            return c as u32;
        }
        if c.is_ascii_lowercase() {
            return c.to_ascii_uppercase() as u32;
        }
        if c.is_ascii_digit() {
            return c as u32;
        }
    }
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
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" | "PGUP" => 0x21,
        "PAGEDOWN" | "PGDN" => 0x22,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        _ => 'T' as u32,
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
                    unsafe {
                        PostMessageW(target, WM_USER_HOTKEY, 0, 0);
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
    let vk = parse_vk(key_str);

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

    debug_log("[hotkey] keyboard hook installed successfully\n");

    Ok(())
}

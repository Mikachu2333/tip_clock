#![allow(clippy::upper_case_acronyms)]

use std::sync::atomic::{AtomicIsize, Ordering};

type HWND = *mut std::ffi::c_void;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

const WM_HOTKEY: u32 = 0x0312;
const WM_DESTROY: u32 = 0x0002;
const HOTKEY_ID: i32 = 1;

/// Custom message posted to the main window when hotkey fires.
pub const WM_USER_HOTKEY: u32 = 0x0401;

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn PostMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut std::ffi::c_void;
}

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut std::ffi::c_void,
    h_icon: *mut std::ffi::c_void,
    h_cursor: *mut std::ffi::c_void,
    hbr_background: *mut std::ffi::c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: *mut std::ffi::c_void,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Parse modifier string like "ctrl+alt" into modifier flags.
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
        flags = MOD_CONTROL | MOD_ALT;
    }
    flags | MOD_NOREPEAT
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

// ── Hidden hotkey window ───────────────────────

static HOTKEY_HWND: AtomicIsize = AtomicIsize::new(0);
static TARGET_HWND: AtomicIsize = AtomicIsize::new(0);

unsafe extern "system" fn hotkey_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if msg == WM_HOTKEY && wparam == HOTKEY_ID as usize {
        let target = TARGET_HWND.load(Ordering::Relaxed) as HWND;
        if !target.is_null() {
            unsafe {
                PostMessageW(target, WM_USER_HOTKEY, 0, 0);
            }
        }
        return 0;
    }
    if msg == WM_DESTROY {
        unsafe {
            UnregisterHotKey(hwnd, HOTKEY_ID);
        }
        return 0;
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn init(mod_str: &str, key_str: &str, target_hwnd: HWND) -> Result<(), String> {
    TARGET_HWND.store(target_hwnd as isize, Ordering::Relaxed);

    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    if hinst.is_null() {
        return Err("GetModuleHandleW failed".into());
    }

    let class_name = to_wide("TipClockHotkeyClass");
    let wc = WNDCLASSEXW {
        cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfn_wnd_proc: hotkey_wndproc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinst,
        h_icon: std::ptr::null_mut(),
        h_cursor: std::ptr::null_mut(),
        hbr_background: std::ptr::null_mut(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
        h_icon_sm: std::ptr::null_mut(),
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err("RegisterClassExW failed".into());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        return Err("CreateWindowExW failed".into());
    }

    HOTKEY_HWND.store(hwnd as isize, Ordering::Relaxed);

    let mods = parse_modifiers(mod_str);
    let vk = parse_vk(key_str);
    let result = unsafe { RegisterHotKey(hwnd, HOTKEY_ID, mods, vk) };
    if result == 0 {
        return Err(format!("RegisterHotKey failed (mod={mods:#x}, vk={vk:#x})"));
    }

    Ok(())
}

/// Re-register with new modifiers/key (after config reload).
#[allow(dead_code)]
pub fn update(mod_str: &str, key_str: &str) {
    let hwnd = HOTKEY_HWND.load(Ordering::Relaxed) as HWND;
    if hwnd.is_null() {
        return;
    }
    unsafe {
        UnregisterHotKey(hwnd, HOTKEY_ID);
        let mods = parse_modifiers(mod_str);
        let vk = parse_vk(key_str);
        RegisterHotKey(hwnd, HOTKEY_ID, mods, vk);
    }
}

#[allow(dead_code)]
pub fn destroy() {
    let hwnd = HOTKEY_HWND.load(Ordering::Relaxed) as HWND;
    if !hwnd.is_null() {
        unsafe {
            DestroyWindow(hwnd);
        }
    }
}

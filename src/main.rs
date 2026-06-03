#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod audio;
mod config;

use std::sync::OnceLock;
use std::time::Duration;

use chrono::{Local, Timelike};
use single_instance::SingleInstance;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use audio::AudioPlayer;
use config::Config;

// ---------------------------------------------------------------------------
// Direct Win32 FFI — user32.dll
// ---------------------------------------------------------------------------

const MB_OK: u32 = 0x0000_0000;
const MB_ICONERROR: u32 = 0x0000_0010;
const PM_REMOVE: u32 = 1;
const QS_ALLINPUT: u32 = 0x04FF;
const WM_QUIT: u32 = 0x0012;

#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
struct MSG {
    hwnd: *mut std::ffi::c_void,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: [i32; 2],
}

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(
        hwnd: *mut std::ffi::c_void,
        text: *const u16,
        caption: *const u16,
        utype: u32,
    ) -> i32;

    fn PeekMessageW(
        msg: *mut MSG,
        hwnd: *mut std::ffi::c_void,
        msg_filter_min: u32,
        msg_filter_max: u32,
        remove_msg: u32,
    ) -> i32;

    fn TranslateMessage(msg: *const MSG) -> i32;

    fn DispatchMessageW(msg: *const MSG) -> isize;

    fn MsgWaitForMultipleObjects(
        count: u32,
        handles: *const *mut std::ffi::c_void,
        wait_all: i32,
        milliseconds: u32,
        wake_mask: u32,
    ) -> u32;
}

/// Show an error popup and exit immediately.
fn fatal(msg: &str) -> ! {
    let text = audio::to_wide(msg);
    let caption = audio::to_wide("Tip Clock — Fatal Error");
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
    std::process::exit(1);
}

// ---------------------------------------------------------------------------
// Global state — initialised synchronously in main().
// ---------------------------------------------------------------------------

static CONFIG: OnceLock<Config> = OnceLock::new();
static AUDIO: OnceLock<AudioPlayer> = OnceLock::new();

// ---------------------------------------------------------------------------
// Schedule helpers
// ---------------------------------------------------------------------------

fn next_label() -> String {
    let now = Local::now();
    let cfg = CONFIG.get().unwrap();
    match cfg.next_reminder(now.hour(), now.minute()) {
        Some((h, m, ring, _)) => format!("Next  {:02}:{:02}  ({})", h, m, ring.display_name()),
        None => "No more reminders today".into(),
    }
}

// ---------------------------------------------------------------------------
// Windows message pump
// ---------------------------------------------------------------------------

/// Drain all pending Windows messages so tray-icon's hidden window
/// receives `WM_USER_TRAYICON` callbacks from `Shell_NotifyIcon`.
fn pump_messages() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                break;
            }
            if msg.message == WM_QUIT {
                std::process::exit(0);
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let instance = SingleInstance::new("6C682EA23C8753664AAD9A6198C672AD").unwrap();
    if !instance.is_single() {
        std::process::exit(1);
    }

    // ---- 1. Load config synchronously (no lazy init, no threads) ----------
    let config = Config::load_or_create().unwrap_or_else(|e| fatal(&e.to_string()));
    CONFIG.set(config).ok();

    // ---- 2. Init audio player ---------------------------------------------
    let audio = AudioPlayer::new();
    AUDIO.set(audio).ok();

    let cfg = CONFIG.get().unwrap();
    let audio = AUDIO.get().unwrap();

    // ---- 3. Event handlers (set before tray creation) -------------------

    // Right-click menu "Exit" → quit.
    MenuEvent::set_event_handler(Some(Box::new(|event: MenuEvent| {
        if event.id == "exit" {
            std::process::exit(0);
        }
    })));

    // ---- 4. Tray right-click context menu --------------------------

    let next_item = MenuItem::new(next_label(), false, None); // disabled → display only
    let exit_item = MenuItem::with_id("exit", "Exit", true, None); // enabled
    let separator = PredefinedMenuItem::separator();

    let menu = Menu::new();
    menu.append(&next_item).ok();
    menu.append(&separator).ok();
    menu.append(&exit_item).ok();

    let icon = {
        let mut rgba = Vec::with_capacity(16 * 16 * 4);
        for _ in 0..16 * 16 {
            rgba.extend_from_slice(&[66u8, 209, 160, 255]);
        }
        tray_icon::Icon::from_rgba(rgba, 16, 16).expect("icon")
    };

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Tip Clock")
        .with_menu(Box::new(menu))
        .with_menu_on_right_click(true)
        .build()
        .unwrap_or_else(|e| fatal(&format!("Tray icon: {e}")));

    // ---- 5. Main loop — message pump + time checking ---------------------

    let mut last_played: Option<(u32, u32)> = None;
    let mut last_refresh = std::time::Instant::now();

    loop {
        // Drain all pending Windows messages so tray-icon's hidden window
        // receives WM_USER_TRAYICON callbacks from Shell_NotifyIcon.
        pump_messages();

        // ----- fire matching schedule entries once per minute -----
        let now = Local::now();
        let current = (now.hour(), now.minute());

        if last_played != Some(current) {
            last_played = Some(current);
            // Schedule is sorted by time — stop once we pass the current minute.
            for entry in &cfg.schedule {
                match config::parse_hhmm(&entry.time) {
                    Some(t) if t == current => audio.play(entry.ring),
                    Some(t) if t > current => break,
                    _ => {}
                }
            }
        }

        // ----- refresh tooltip + menu text every ~30 s -----
        if last_refresh.elapsed() >= Duration::from_secs(30) {
            last_refresh = std::time::Instant::now();
            let label = next_label();
            tray.set_tooltip(Some(&label)).ok();
            next_item.set_text(&label);
        }

        // Wait for new Windows messages, or wake every N seconds to
        // check the schedule.  QS_ALLINPUT wakes on any message.
        let timeout_ms = (cfg.interval_secs.max(10)) * 1000;
        unsafe {
            MsgWaitForMultipleObjects(0, std::ptr::null(), 0, timeout_ms, QS_ALLINPUT);
        }
    }
}

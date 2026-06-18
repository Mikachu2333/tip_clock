#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod audio;
mod config;

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use chrono::{Local, Timelike};
use single_instance::SingleInstance;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use audio::AudioPlayer;
use config::Config;

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

    fn SetProcessDPIAware() -> i32;
}

const PROCESS_PER_MONITOR_DPI_AWARE: i32 = 2;

#[link(name = "shcore")]
unsafe extern "system" {
    fn SetProcessDpiAwareness(value: i32) -> i32;
}

fn set_dpi_aware() {
    unsafe {
        if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) != 0 {
            let _ = SetProcessDPIAware();
        }
    }
}

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

static CONFIG: OnceLock<Config> = OnceLock::new();
static AUDIO: OnceLock<AudioPlayer> = OnceLock::new();
static SKIP_COUNT: OnceLock<AtomicU32> = OnceLock::new();
static PAUSED: OnceLock<AtomicBool> = OnceLock::new();
static NEED_REFRESH: OnceLock<AtomicBool> = OnceLock::new();

fn next_label() -> String {
    let now = Local::now();
    let cfg = CONFIG.get().unwrap();
    match cfg.next_reminder(now.hour(), now.minute()) {
        Some((h, m, ring)) => format!("Next  {:02}:{:02}  ({})", h, m, ring.display_name()),
        None => "No more reminders today".into(),
    }
}

fn next_after_skip_label() -> String {
    let now = Local::now();
    let cfg = CONFIG.get().unwrap();
    let count = SKIP_COUNT.get().unwrap().load(Ordering::Relaxed);

    let mut h = now.hour();
    let mut m = now.minute();

    for i in 0..=count {
        match cfg.next_reminder(h, m) {
            Some((h2, m2, ring)) => {
                h = h2;
                m = m2;
                if i == count {
                    return format!("Next  {:02}:{:02}  ({})", h, m, ring.display_name());
                }
            }
            None => return "No more reminders today".into(),
        }
    }
    "No more reminders today".into()
}

fn refresh_menu_items(
    tray: &TrayIcon,
    next_item: &MenuItem,
    pause_item: &MenuItem,
    skip_item: &MenuItem,
) {
    let paused = PAUSED.get().unwrap().load(Ordering::Relaxed);
    if paused {
        tray.set_tooltip(Some("Tip Clock — Paused")).ok();
        next_item.set_text("None (paused)");
        skip_item.set_enabled(false);
        pause_item.set_text("Resume");
    } else {
        next_item.set_text(&next_label());
        skip_item.set_enabled(true);
        pause_item.set_text("Pause");
        tray.set_tooltip(Some(&next_label())).ok();
    }
}

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

fn main() {
    set_dpi_aware();

    let instance = SingleInstance::new("6C682EA23C8753664AAD9A6198C672AD")
        .unwrap_or_else(|e| fatal(&e.to_string()));
    if !instance.is_single() {
        std::process::exit(1);
    }

    let config = Config::load_or_create().unwrap_or_else(|e| fatal(&e.to_string()));
    CONFIG.set(config).ok();

    let audio = AudioPlayer::new();
    AUDIO.set(audio).ok();

    SKIP_COUNT.set(AtomicU32::new(0)).ok();
    PAUSED.set(AtomicBool::new(false)).ok();
    NEED_REFRESH.set(AtomicBool::new(false)).ok();

    let cfg = CONFIG.get().unwrap();
    let audio = AUDIO.get().unwrap();

    let next_item = MenuItem::new(next_label(), false, None);
    let skip_item = MenuItem::with_id("skip_next", "Skip next", true, None);
    let pause_item = MenuItem::with_id("toggle_pause", "Pause", true, None);
    let exit_item = MenuItem::with_id("exit", "Exit", true, None);
    let sep = PredefinedMenuItem::separator();

    MenuEvent::set_event_handler(Some(Box::new(|event: MenuEvent| match event.id.as_ref() {
        "exit" => std::process::exit(0),
        "skip_next" => {
            SKIP_COUNT.get().unwrap().fetch_add(1, Ordering::Relaxed);
            NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
        }
        "toggle_pause" => {
            let paused = PAUSED.get().unwrap();
            paused.store(!paused.load(Ordering::Relaxed), Ordering::Relaxed);
            NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
        }
        _ => {}
    })));

    let menu = Menu::new();
    menu.append(&next_item).ok();
    menu.append(&sep).ok();
    menu.append(&skip_item).ok();
    menu.append(&pause_item).ok();
    menu.append(&sep).ok();
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

    let mut last_played: Option<(u32, u32)> = None;
    let mut last_refresh = std::time::Instant::now();

    loop {
        pump_messages();

        // Immediate menu refresh when state changes (skip / pause toggled).
        if NEED_REFRESH.get().unwrap().swap(false, Ordering::Relaxed) {
            if PAUSED.get().unwrap().load(Ordering::Relaxed) {
                refresh_menu_items(&tray, &next_item, &pause_item, &skip_item);
            } else if SKIP_COUNT.get().unwrap().load(Ordering::Relaxed) > 0 {
                let label = next_after_skip_label();
                tray.set_tooltip(Some(&label)).ok();
                next_item.set_text(&label);
                skip_item.set_enabled(true);
                pause_item.set_text("Pause");
            } else {
                refresh_menu_items(&tray, &next_item, &pause_item, &skip_item);
            }
            last_refresh = std::time::Instant::now();
        }

        let now = Local::now();
        let current = (now.hour(), now.minute());

        if last_played != Some(current) {
            last_played = Some(current);

            let paused = PAUSED.get().unwrap().load(Ordering::Relaxed);
            let do_skip = if paused {
                true
            } else {
                let has_match = cfg
                    .schedule
                    .iter()
                    .any(|e| config::parse_hhmm(&e.time) == Some(current));
                if has_match {
                    let count = SKIP_COUNT.get().unwrap();
                    let prev = count.fetch_update(
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                        |v| if v > 0 { Some(v - 1) } else { None },
                    );
                    prev.is_ok()
                } else {
                    false
                }
            };

            if !do_skip {
                for entry in &cfg.schedule {
                    match config::parse_hhmm(&entry.time) {
                        Some(t) if t == current => audio.play(entry.ring),
                        Some(t) if t > current => break,
                        _ => {}
                    }
                }
            }

            // Minute changed — refresh menu in case skip was consumed.
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item);
        }

        if last_refresh.elapsed() >= Duration::from_secs(30) {
            last_refresh = std::time::Instant::now();
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item);
        }

        let timeout_ms = (cfg.interval_secs.max(10)) * 1000;
        unsafe {
            MsgWaitForMultipleObjects(0, std::ptr::null(), 0, timeout_ms, QS_ALLINPUT);
        }
    }
}

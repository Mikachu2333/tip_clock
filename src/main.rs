#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(clippy::upper_case_acronyms)]

mod audio;
mod config;
mod gui;
mod hotkey;
mod i18n;

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
use config::{Config, RingType};

const DEBUG_MODE: bool = cfg!(debug_assertions);
const PROCESS_GUID: &str = "F44E29E669346E0CC3105EA440E85C00";

// ───────────────────────────────────────────────
//  Win32 constants & types
// ───────────────────────────────────────────────

const MB_OK: u32 = 0x0000_0000;
const MB_ICONERROR: u32 = 0x0000_0010;
const INTERVAL_SECS: u32 = 15;
const PM_REMOVE: u32 = 1;
const QS_ALLINPUT: u32 = 0x04FF;
const WM_QUIT: u32 = 0x0012;

type HWND = *mut std::ffi::c_void;

// ───────────────────────────────────────────────
//  ChooseFont / ChooseColor dialog structures
// ───────────────────────────────────────────────


#[allow(dead_code)]
#[repr(C)]
struct CHOOSECOLORW {
    l_struct_size: u32,
    hwnd_owner: HWND,
    h_instance: *mut std::ffi::c_void,
    rgb_result: u32,
    lp_cust_colors: *mut u32,
    flags: u32,
    l_cust_data: isize,
    lpfn_hook: *mut std::ffi::c_void,
    lp_template_name: *const u16,
}

const CC_RGBINIT: u32 = 0x0000_0001;
const CC_FULLOPEN: u32 = 0x0000_0002;

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn ChooseColorW(lpcc: *mut CHOOSECOLORW) -> i32;
}

// ───────────────────────────────────────────────
//  Message struct
// ───────────────────────────────────────────────

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: [i32; 2],
}

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, utype: u32) -> i32;

    fn PeekMessageW(
        msg: *mut MSG,
        hwnd: HWND,
        msg_filter_min: u32,
        msg_filter_max: u32,
        remove_msg: u32,
    ) -> i32;

    fn TranslateMessage(msg: *const MSG) -> i32;

    fn DispatchMessageW(msg: *const MSG) -> isize;

    fn MsgWaitForMultipleObjects(
        count: u32,
        handles: *const HWND,
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

#[link(name = "kernel32")]
unsafe extern "system" {
    fn AttachConsole(dwProcessId: u32) -> i32;
}

fn try_attach_console() {
    unsafe {
        AttachConsole(0xFFFF_FFFF); // ATTACH_PARENT_PROCESS
    }
}

fn debug_log(s: &str) {
    if DEBUG_MODE {
        audio::debug_log(s);
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

// ───────────────────────────────────────────────
//  Global state
// ───────────────────────────────────────────────

static CONFIG: OnceLock<std::sync::Mutex<Config>> = OnceLock::new();
static AUDIO: OnceLock<AudioPlayer> = OnceLock::new();
static SKIP_COUNT: OnceLock<AtomicU32> = OnceLock::new();
static PAUSED: OnceLock<AtomicBool> = OnceLock::new();
static NEED_REFRESH: OnceLock<AtomicBool> = OnceLock::new();

// ───────────────────────────────────────────────
//  Menu helpers
// ───────────────────────────────────────────────

fn next_label(cfg: &Config) -> String {
    let now = Local::now();
    match cfg.next_reminder(now.hour(), now.minute()) {
        Some((h, m, ring)) => {
            format!(
                "{}  {:02}:{:02}  ({})",
                i18n::tr(i18n::TrKey::NextReminder),
                h,
                m,
                ring.display_name()
            )
        }
        None => i18n::tr(i18n::TrKey::NoMoreReminders).to_string(),
    }
}

#[allow(dead_code)]
fn next_after_skip_label(cfg: &Config) -> String {
    let now = Local::now();
    let count = SKIP_COUNT.get().unwrap().load(Ordering::Relaxed);

    let mut h = now.hour();
    let mut m = now.minute();

    for i in 0..=count {
        match cfg.next_reminder(h, m) {
            Some((h2, m2, ring)) => {
                h = h2;
                m = m2;
                if i == count {
                    return format!(
                        "{}  {:02}:{:02}  ({})",
                        i18n::tr(i18n::TrKey::NextReminder),
                        h,
                        m,
                        ring.display_name()
                    );
                }
            }
            None => return i18n::tr(i18n::TrKey::NoMoreReminders).to_string(),
        }
    }
    i18n::tr(i18n::TrKey::NoMoreReminders).to_string()
}

fn refresh_menu_items(
    tray: &TrayIcon,
    next_item: &MenuItem,
    pause_item: &MenuItem,
    skip_item: &MenuItem,
    show_item: &MenuItem,
) {
    let cfg = CONFIG.get().unwrap().lock().unwrap();
    let paused = PAUSED.get().unwrap().load(Ordering::Relaxed);

    if paused {
        tray.set_tooltip(Some(&format!(
            "{} — {}",
            i18n::tr(i18n::TrKey::AppName),
            i18n::tr(i18n::TrKey::Pause)
        )))
        .ok();
        next_item.set_text(i18n::tr(i18n::TrKey::NoMoreReminders));
        skip_item.set_enabled(false);
        pause_item.set_text(i18n::tr(i18n::TrKey::Resume));
    } else {
        let label = next_label(&cfg);
        tray.set_tooltip(Some(&label)).ok();
        next_item.set_text(&label);
        skip_item.set_enabled(true);
        pause_item.set_text(i18n::tr(i18n::TrKey::Pause));
    }

    show_item.set_text(if gui::is_visible() {
        format!("{} (visible)", i18n::tr(i18n::TrKey::ShowClock))
    } else {
        i18n::tr(i18n::TrKey::ShowClock).to_string()
    });
}

// ───────────────────────────────────────────────
//  Message pump
// ───────────────────────────────────────────────

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
            if msg.message != 0x0113 {
                // Log every dispatched message with hwnd and message code
                debug_log(&format!(
                    "[main] pump: hwnd={:?} msg=0x{:04x}\n",
                    msg.hwnd, msg.message
                ));
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// ───────────────────────────────────────────────
//  Auto-start — Windows registry Run key
// ───────────────────────────────────────────────

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegOpenKeyExW(
        hKey: *mut std::ffi::c_void,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut *mut std::ffi::c_void,
    ) -> i32;

    fn RegSetValueExW(
        hKey: *mut std::ffi::c_void,
        lpValueName: *const u16,
        reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> i32;

    fn RegDeleteValueW(hKey: *mut std::ffi::c_void, lpValueName: *const u16) -> i32;

    fn RegCloseKey(hKey: *mut std::ffi::c_void) -> i32;
}

const HKEY_CURRENT_USER: *mut std::ffi::c_void = 0x8000_0001usize as *mut std::ffi::c_void;
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_QUERY_VALUE: u32 = 0x0001;
const REG_SZ: u32 = 1;
const ERROR_SUCCESS: i32 = 0;

fn update_auto_start(enable: bool) {
    unsafe {
        let sub_key = audio::to_wide("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run");
        let value_name = audio::to_wide("TipClock");
        let mut hkey: *mut std::ffi::c_void = std::ptr::null_mut();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );

        if result != ERROR_SUCCESS {
            return;
        }

        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path_str = format!("{}", exe_path.display());
                let wide_path = audio::to_wide(&path_str);
                RegSetValueExW(
                    hkey,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    wide_path.as_ptr() as *const u8,
                    (wide_path.len() * 2) as u32, // bytes including null
                );
            }
        } else {
            RegDeleteValueW(hkey, value_name.as_ptr());
        }

        RegCloseKey(hkey);
    }
}

// ───────────────────────────────────────────────
//  Font & color dialogs
// ───────────────────────────────────────────────

fn dialog_choose_color() {
    let cfg = CONFIG.get().unwrap().lock().unwrap();
    let rgb: u32 = (cfg.general.text_r as u32)
        | ((cfg.general.text_g as u32) << 8)
        | ((cfg.general.text_b as u32) << 16);
    drop(cfg);

    let mut cust_colors: [u32; 16] = [0; 16];
    let mut cc = CHOOSECOLORW {
        l_struct_size: std::mem::size_of::<CHOOSECOLORW>() as u32,
        hwnd_owner: std::ptr::null_mut(),
        h_instance: std::ptr::null_mut(),
        rgb_result: rgb,
        lp_cust_colors: cust_colors.as_mut_ptr(),
        flags: CC_RGBINIT | CC_FULLOPEN,
        l_cust_data: 0,
        lpfn_hook: std::ptr::null_mut(),
        lp_template_name: std::ptr::null(),
    };

    let result = unsafe { ChooseColorW(&mut cc) };
    if result != 0 {
        let r = (cc.rgb_result & 0xFF) as u8;
        let g = ((cc.rgb_result >> 8) & 0xFF) as u8;
        let b = ((cc.rgb_result >> 16) & 0xFF) as u8;

        let mut cfg = CONFIG.get().unwrap().lock().unwrap();
        cfg.general.text_r = r;
        cfg.general.text_g = g;
        cfg.general.text_b = b;
        let _ = cfg.save_to_file();
        drop(cfg);
        gui::update_config(&CONFIG.get().unwrap().lock().unwrap().general);
    }
}

fn dialog_choose_bg_color() {
    let cfg = CONFIG.get().unwrap().lock().unwrap();
    let rgb: u32 = (cfg.general.bg_r as u32)
        | ((cfg.general.bg_g as u32) << 8)
        | ((cfg.general.bg_b as u32) << 16);
    drop(cfg);

    let mut cust_colors: [u32; 16] = [0; 16];
    let mut cc = CHOOSECOLORW {
        l_struct_size: std::mem::size_of::<CHOOSECOLORW>() as u32,
        hwnd_owner: std::ptr::null_mut(),
        h_instance: std::ptr::null_mut(),
        rgb_result: rgb,
        lp_cust_colors: cust_colors.as_mut_ptr(),
        flags: CC_RGBINIT | CC_FULLOPEN,
        l_cust_data: 0,
        lpfn_hook: std::ptr::null_mut(),
        lp_template_name: std::ptr::null(),
    };

    let result = unsafe { ChooseColorW(&mut cc) };
    if result != 0 {
        let r = (cc.rgb_result & 0xFF) as u8;
        let g = ((cc.rgb_result >> 8) & 0xFF) as u8;
        let b = ((cc.rgb_result >> 16) & 0xFF) as u8;

        let mut cfg = CONFIG.get().unwrap().lock().unwrap();
        cfg.general.bg_r = r;
        cfg.general.bg_g = g;
        cfg.general.bg_b = b;
        let _ = cfg.save_to_file();
        drop(cfg);
        gui::update_config(&CONFIG.get().unwrap().lock().unwrap().general);
    }
}

/// Create a 16×16 solid-color tray icon from R, G, B values (0-255).
fn make_tray_icon_rgba(r: u8, g: u8, b: u8) -> tray_icon::Icon {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..16 * 16 {
        rgba.extend_from_slice(&[r, g, b, 255]);
    }
    tray_icon::Icon::from_rgba(rgba, 16, 16).expect("icon")
}

// ───────────────────────────────────────────────
//  Main
// ───────────────────────────────────────────────

fn main() {
    try_attach_console();
    set_dpi_aware();

    // Single instance
    let instance = SingleInstance::new(PROCESS_GUID).unwrap_or_else(|e| fatal(&e.to_string()));
    if !instance.is_single() {
        std::process::exit(1);
    }

    // i18n
    i18n::init();

    // Config
    let config = Config::load_or_create().unwrap_or_else(|e| fatal(&e));
    if DEBUG_MODE {
        dbg!(&config);
    }

    // Apply auto-start setting
    update_auto_start(config.general.auto_start);

    let app_name = i18n::tr(i18n::TrKey::AppName);

    // Audio
    let audio = AudioPlayer::new(config.general.volume);

    // Create GUI clock window (hidden)
    gui::create_clock_window(&config.general).unwrap_or_else(|e| fatal(&e));
    let gui_hwnd = gui::get_hwnd();

    // Hotkey — dedicated hidden window for reliable WM_HOTKEY delivery
    hotkey::init(
        &config.general.hotkey_mod,
        &config.general.hotkey_key,
        gui_hwnd,
    )
    .map(|()| {
        debug_log(&format!(
            "[main] hotkey registered: {}+{}\n",
            config.general.hotkey_mod, config.general.hotkey_key
        ));
    })
    .unwrap_or_else(|e| {
        audio::debug_log(&format!("[tip_clock] hotkey init failed: {e}\n"));
    });

    CONFIG.set(std::sync::Mutex::new(config)).ok();
    AUDIO.set(audio).ok();
    SKIP_COUNT.set(AtomicU32::new(0)).ok();
    PAUSED.set(AtomicBool::new(false)).ok();
    NEED_REFRESH.set(AtomicBool::new(false)).ok();

    // ── Build tray menu ────────────────────────

    let next_item = MenuItem::new(
        next_label(&CONFIG.get().unwrap().lock().unwrap()),
        false,
        None,
    );
    let show_item = MenuItem::with_id("show_clock", i18n::tr(i18n::TrKey::ShowClock), true, None);
    let skip_item = MenuItem::with_id("skip_next", i18n::tr(i18n::TrKey::SkipNext), true, None);
    let pause_item = MenuItem::with_id("toggle_pause", i18n::tr(i18n::TrKey::Pause), true, None);
    let edit_item = MenuItem::with_id("edit_config", i18n::tr(i18n::TrKey::EditConfig), true, None);
    let color_item = MenuItem::with_id("text_color", i18n::tr(i18n::TrKey::TextColor), true, None);
    let bg_color_item = MenuItem::with_id("bg_color", i18n::tr(i18n::TrKey::BgColor), true, None);
    let exit_item = MenuItem::with_id("exit", i18n::tr(i18n::TrKey::Exit), true, None);
    let sep = PredefinedMenuItem::separator();
    let sep2 = PredefinedMenuItem::separator();
    let sep3 = PredefinedMenuItem::separator();
    let sep4 = PredefinedMenuItem::separator();

    MenuEvent::set_event_handler(Some(Box::new(|event: MenuEvent| {
        debug_log(&format!("[main] tray menu clicked: {:?}\n", event.id));
        match event.id.as_ref() {
            "exit" => {
                debug_log("[main] tray menu: exit\n");
                std::process::exit(0);
            }
            "skip_next" => {
                debug_log("[main] tray menu: skip next\n");
                SKIP_COUNT.get().unwrap().fetch_add(1, Ordering::Relaxed);
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "toggle_pause" => {
                debug_log("[main] tray menu: toggle pause\n");
                PAUSED.get().unwrap().fetch_not(Ordering::Relaxed);
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "show_clock" => {
                debug_log("[main] tray menu: show/hide clock\n");
                if gui::is_visible() {
                    gui::hide_clock();
                } else {
                    gui::show_clock();
                }
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "edit_config" => {
                debug_log("[main] tray menu: edit config\n");
                let cfg = CONFIG.get().unwrap().lock().unwrap();
                let path = cfg.config_path.clone();
                drop(cfg);
                let _ = std::process::Command::new("notepad.exe").arg(&path).spawn();
            }
            "text_color" => {
                debug_log("[main] tray menu: text color\n");
                dialog_choose_color();
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "bg_color" => {
                debug_log("[main] tray menu: background color\n");
                dialog_choose_bg_color();
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            _ => {
                debug_log(&format!("[main] tray menu: unknown id '{:?}'\n", event.id));
            }
        }
    })));

    let menu = Menu::new();
    menu.append(&next_item).ok();
    menu.append(&sep).ok();
    menu.append(&show_item).ok();
    menu.append(&skip_item).ok();
    menu.append(&pause_item).ok();
    menu.append(&sep2).ok();
    menu.append(&edit_item).ok();
    menu.append(&sep3).ok();
    menu.append(&color_item).ok();
    menu.append(&bg_color_item).ok();
    menu.append(&sep4).ok();
    menu.append(&exit_item).ok();

    // Create tray icon — solid color 16×16
    let icon = make_tray_icon_rgba(61, 176, 87);

    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip(app_name)
        .with_menu(Box::new(menu))
        .build()
        .unwrap_or_else(|e| fatal(&format!("Tray icon: {e}")));
    debug_log("[main] tray icon created\n");

    // Left click → toggle clock; right click → show menu.
    tray.set_show_menu_on_left_click(false);
    debug_log("[main] tray: left-click menu disabled, right-click menu enabled\n");

    // TrayIconEvent for left-click toggle. Only matching Left button ensures
    // right-click still triggers the context menu via the default handling.
    tray_icon::TrayIconEvent::set_event_handler(Some(Box::new(
        |event: tray_icon::TrayIconEvent| {
            if let tray_icon::TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                ..
            } = event
            {
                debug_log("[main] tray left-click: toggle clock\n");
                if gui::is_visible() {
                    gui::hide_clock();
                } else {
                    gui::show_clock();
                }
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
        },
    )));

    // ── Main loop ──────────────────────────────

    let mut last_played_second: Option<(u32, u32, u32)> = None;
    let mut last_menu_refresh = std::time::Instant::now();

    // Initial menu refresh
    refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);

    debug_log("[main] entering main loop\n");
    let mut loop_count: u64 = 0;

    loop {
        loop_count += 1;
        if loop_count % 120 == 1 {
            // Heartbeat roughly every 60 seconds (120 iterations × 500ms)
            debug_log(&format!("[main] heartbeat: iteration {loop_count}\n"));
        }
        pump_messages();

        // WM_USER_HOTKEY is posted by the keyboard hook in hotkey.rs
        // and dispatched to the clock window via pump_messages above.
        let now = Local::now();
        let current = (now.hour(), now.minute(), now.second());

        // Menu state refresh (immediate if needed, otherwise periodic)
        if NEED_REFRESH.get().unwrap().swap(false, Ordering::Relaxed) {
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);
            last_menu_refresh = std::time::Instant::now();
        }

        // Schedule check: every second
        if last_played_second != Some(current) {
            last_played_second = Some(current);

            let paused = PAUSED.get().unwrap().load(Ordering::Relaxed);

            // Scope the config lock so it's released before refresh_menu_items (avoids deadlock)
            let matches_found = {
                let cfg = CONFIG.get().unwrap().lock().unwrap();

                // Determine if we should skip the next match
                let do_skip = if paused {
                    true
                } else {
                    let matches = cfg.entries_at(current.0, current.1, current.2);
                    if !matches.is_empty() {
                        let count = SKIP_COUNT.get().unwrap();
                        let prev = count.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                            if v > 0 { Some(v - 1) } else { None }
                        });
                        prev.is_ok()
                    } else {
                        false
                    }
                };

                if !do_skip {
                    let entries = cfg.entries_at(current.0, current.1, current.2);
                    for entry in entries {
                        if entry.ring != RingType::None {
                            debug_log(&format!(
                                "[main] schedule match at {:02}:{:02}:{:02}, ring={:?}\n",
                                current.0, current.1, current.2, entry.ring
                            ));
                            AUDIO.get().unwrap().play(
                                entry.ring,
                                entry.custom_file.as_deref(),
                                &cfg.exe_dir,
                            );
                            gui::show_clock();
                        }
                    }
                    true
                } else {
                    false
                }
            }; // cfg lock released here

            // Refresh menu after minute change (lock-free now)
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);
            last_menu_refresh = std::time::Instant::now();

            if matches_found {
                debug_log("[main] schedule match processed\n");
            }
        }

        // Periodic menu refresh (every INTERVAL_SECS)
        if last_menu_refresh.elapsed() >= Duration::from_secs(INTERVAL_SECS as u64) {
            last_menu_refresh = std::time::Instant::now();
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);
        }

        // Wait for next message or timeout
        unsafe {
            MsgWaitForMultipleObjects(0, std::ptr::null(), 0, 500, QS_ALLINPUT);
        }
    }
}

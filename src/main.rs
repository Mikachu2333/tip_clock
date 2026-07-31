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
use config::Config;

const PROCESS_GUID: &str = "F44E29E669346E0CC3105EA440E85C00";

// ───────────────────────────────────────────────
//  Win32 constants & types
// ───────────────────────────────────────────────

const INTERVAL_SECS: u32 = 15;
const PM_REMOVE: u32 = 1;
const QS_ALLINPUT: u32 = 0x04FF;
const WM_QUIT: u32 = 0x0012;
const WM_TIMER: u32 = 0x0113;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_NCMOUSEMOVE: u32 = 0x00A0;

type HWND = *mut std::ffi::c_void;

// ───────────────────────────────────────────────
//  ChooseFont / ChooseColor dialog structures
// ───────────────────────────────────────────────

enum ColorKind {
    Text,
    Bg,
}

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
    l_private: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
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
    fn SetProcessDpiAwarenessContext(value: isize) -> i32;
}

const PROCESS_PER_MONITOR_DPI_AWARE: i32 = 2;
// DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

#[link(name = "shcore")]
unsafe extern "system" {
    fn SetProcessDpiAwareness(value: i32) -> i32;
}

fn set_dpi_aware() {
    // SAFETY: process DPI awareness is configured once, before any HWND exists.
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0
            && SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) != 0
        {
            SetProcessDPIAware();
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

fn debug_log(s: impl ToString) {
    audio::debug_log(s);
}

fn fatal(msg: &str) -> ! {
    win_msgbox_timeout::error_msgbox(msg, "Tip Clock — Fatal Error", 0);
    std::process::exit(1);
}

fn report_error(context: &str, error: impl std::fmt::Display) {
    let message = format!("{context}: {error}");
    debug_log(format!("[main] {message}\n"));
    win_msgbox_timeout::error_msgbox(&message, "Tip Clock", 10);
}

fn shutdown_runtime() {
    hotkey::shutdown();
    gui::destroy_windows();
}

fn quit_app(code: i32) -> ! {
    shutdown_runtime();
    std::process::exit(code);
}

fn restart_app() {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            debug_log(format!(
                "[main] cannot locate executable for restart: {e}\n"
            ));
            return;
        }
    };
    debug_log(format!("[main] restarting: {:?}\n", exe_path));
    shutdown_runtime();
    if let Some(mtx) = INSTANCE.get() {
        mtx.lock().unwrap_or_else(|e| e.into_inner()).take();
    }
    match std::process::Command::new(&exe_path).spawn() {
        Ok(_) => std::process::exit(0),
        Err(e) => fatal(&format!("Failed to restart Tip Clock: {e}")),
    }
}

// ───────────────────────────────────────────────
//  Global state
// ───────────────────────────────────────────────

static CONFIG: OnceLock<std::sync::Mutex<Config>> = OnceLock::new();
static AUDIO: OnceLock<AudioPlayer> = OnceLock::new();
static SKIP_COUNT: OnceLock<AtomicU32> = OnceLock::new();
static PAUSED: OnceLock<AtomicBool> = OnceLock::new();
static NEED_REFRESH: OnceLock<AtomicBool> = OnceLock::new();
static INSTANCE: OnceLock<std::sync::Mutex<Option<SingleInstance>>> = OnceLock::new();

// ───────────────────────────────────────────────
//  Menu helpers
// ───────────────────────────────────────────────

fn next_label(cfg: &Config, skip_count: u32) -> String {
    let now = Local::now();
    let current_sec = now.hour() * 3600 + now.minute() * 60 + now.second();

    // A reminder group is every entry sharing one timestamp. Build today's
    // remaining groups followed by tomorrow's groups so skip_count has exactly
    // the same group semantics as the scheduler.
    let mut groups: Vec<(u32, bool)> = cfg
        .entries
        .iter()
        .filter(|entry| entry.total_sec > current_sec)
        .map(|entry| (entry.total_sec, false))
        .collect();
    groups.dedup_by_key(|group| group.0);

    let mut tomorrow: Vec<(u32, bool)> = cfg
        .entries
        .iter()
        .map(|entry| (entry.total_sec, true))
        .collect();
    tomorrow.dedup_by_key(|group| group.0);
    groups.extend(tomorrow);

    let Some((total_sec, tomorrow)) = groups.get(skip_count as usize).copied() else {
        return i18n::tr(i18n::TrKey::NoMoreReminders).to_string();
    };
    let has_audio = cfg
        .entries
        .iter()
        .any(|entry| entry.total_sec == total_sec && entry.audio.is_some());
    let day = if tomorrow {
        i18n::tr(i18n::TrKey::Tomorrow)
    } else {
        ""
    };
    format!(
        "{}  {}{:02}:{:02}:{:02}{}",
        i18n::tr(i18n::TrKey::NextReminder),
        day,
        total_sec / 3600,
        total_sec / 60 % 60,
        total_sec % 60,
        if has_audio { "  ♪" } else { "" }
    )
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
    let skip_count = SKIP_COUNT.get().unwrap().load(Ordering::Relaxed);

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
        let label = next_label(&cfg, skip_count);
        tray.set_tooltip(Some(&label)).ok();
        next_item.set_text(&label);
        skip_item.set_enabled(true);
        pause_item.set_text(i18n::tr(i18n::TrKey::Pause));
    }

    show_item.set_text(i18n::tr(i18n::TrKey::SHClock));
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
                quit_app(0);
            }
            if !matches!(msg.message, WM_TIMER | WM_MOUSEMOVE | WM_NCMOUSEMOVE) {
                // Exclude high-frequency timer and pointer-motion messages.
                debug_log(format!(
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

fn update_auto_start(enable: bool) -> Result<(), String> {
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
            return Err(format!("RegOpenKeyExW failed with code {result}"));
        }

        let operation = if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {e}"))?;
            // Run-key command lines must quote executable paths containing
            // spaces; quoting unconditionally is safe and unambiguous.
            let wide_path = audio::to_wide(&format!("\"{}\"", exe_path.display()));
            let byte_len = wide_path
                .len()
                .checked_mul(std::mem::size_of::<u16>())
                .and_then(|len| u32::try_from(len).ok())
                .ok_or("Auto-start command is too long")?;
            RegSetValueExW(
                hkey,
                value_name.as_ptr(),
                0,
                REG_SZ,
                wide_path.as_ptr().cast(),
                byte_len,
            )
        } else {
            RegDeleteValueW(hkey, value_name.as_ptr())
        };

        let close_result = RegCloseKey(hkey);
        if operation != ERROR_SUCCESS && !(operation == 2 && !enable) {
            return Err(format!("Failed to update auto-start (code {operation})"));
        }
        if close_result != ERROR_SUCCESS {
            return Err(format!("RegCloseKey failed with code {close_result}"));
        }
        Ok(())
    }
}

// ───────────────────────────────────────────────
//  Font & color dialogs
// ───────────────────────────────────────────────

fn dialog_choose_color(kind: ColorKind) {
    let cfg = CONFIG.get().unwrap().lock().unwrap();
    let (init_r, init_g, init_b) = match kind {
        ColorKind::Text => (cfg.general.text_r, cfg.general.text_g, cfg.general.text_b),
        ColorKind::Bg => (cfg.general.bg_r, cfg.general.bg_g, cfg.general.bg_b),
    };
    let rgb: u32 = (init_r as u32) | ((init_g as u32) << 8) | ((init_b as u32) << 16);
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
        match kind {
            ColorKind::Text => {
                cfg.general.text_r = r;
                cfg.general.text_g = g;
                cfg.general.text_b = b;
            }
            ColorKind::Bg => {
                cfg.general.bg_r = r;
                cfg.general.bg_g = g;
                cfg.general.bg_b = b;
            }
        }
        if let Err(e) = cfg.save_to_file() {
            debug_log(format!("[main] failed to save config: {e}\n"));
        }
        drop(cfg);
        gui::update_config(&CONFIG.get().unwrap().lock().unwrap().general);
    }
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
        std::process::exit(0);
    }
    INSTANCE.set(std::sync::Mutex::new(Some(instance))).ok();

    // i18n
    i18n::init();

    // Config
    let config = Config::load_or_create().unwrap_or_else(|e| fatal(&e));
    debug_log(format!("{:?}", config));

    // Apply auto-start setting. This is non-fatal because reminders can still
    // run, but the failure is made visible in debug diagnostics.
    if let Err(e) = update_auto_start(config.general.auto_start) {
        debug_log(format!("[main] auto-start update failed: {e}\n"));
    }

    let app_name = i18n::tr(i18n::TrKey::AppName);

    // Audio: volume is applied only to this process' output stream.
    let audio = AudioPlayer::new(config.general.volume).unwrap_or_else(|e| fatal(&e));

    // Create GUI clock window (hidden)
    gui::create_clock_window(&config.general).unwrap_or_else(|e| fatal(&e));
    // Create opacity panel (hidden, modeless)
    gui::create_opacity_panel().unwrap_or_else(|e| fatal(&e));
    let gui_hwnd = gui::get_hwnd();

    // OS-managed global hotkey (MOD_NOREPEAT avoids key-repeat toggling).
    hotkey::init(
        &config.general.hotkey_mod,
        &config.general.hotkey_key,
        gui_hwnd,
    )
    .map(|()| {
        debug_log(format!(
            "[main] hotkey registered: {}+{}\n",
            config.general.hotkey_mod, config.general.hotkey_key
        ));
    })
    .unwrap_or_else(|e| {
        audio::debug_log(format!("[tip_clock] hotkey init failed: {e}\n"));
    });

    CONFIG
        .set(std::sync::Mutex::new(config))
        .unwrap_or_else(|_| {
            fatal("Failed to initialize CONFIG: already initialized");
        });
    AUDIO.set(audio).unwrap_or_else(|_| {
        fatal("Failed to initialize AUDIO: already initialized");
    });
    SKIP_COUNT.set(AtomicU32::new(0)).ok();
    PAUSED.set(AtomicBool::new(false)).ok();
    NEED_REFRESH.set(AtomicBool::new(false)).ok();

    // Register callback to persist window position on drag
    gui::set_position_callback(|x, y| {
        if let Some(cfg_lock) = CONFIG.get()
            && let Ok(mut cfg) = cfg_lock.lock()
        {
            cfg.general.window_x = x;
            cfg.general.window_y = y;
            if let Err(e) = cfg.save_to_file() {
                debug_log(format!("[main] failed to save position: {e}\n"));
            }
        }
    });

    // Register callback to persist opacity on panel close
    gui::set_opacity_close_callback(|opacity| {
        if let Some(cfg_lock) = CONFIG.get()
            && let Ok(mut cfg) = cfg_lock.lock()
        {
            cfg.general.bg_opacity = opacity;
            if let Err(e) = cfg.save_to_file() {
                debug_log(format!("[main] failed to save opacity: {e}\n"));
            }
        }
    });

    let next_item = MenuItem::new(
        next_label(&CONFIG.get().unwrap().lock().unwrap(), 0),
        false,
        None,
    );
    let show_item = MenuItem::with_id("show_clock", i18n::tr(i18n::TrKey::SHClock), true, None);
    let skip_item = MenuItem::with_id("skip_next", i18n::tr(i18n::TrKey::SkipNext), true, None);
    let pause_item = MenuItem::with_id("toggle_pause", i18n::tr(i18n::TrKey::Pause), true, None);
    let edit_item = MenuItem::with_id("edit_config", i18n::tr(i18n::TrKey::EditConfig), true, None);
    let color_item = MenuItem::with_id("text_color", i18n::tr(i18n::TrKey::TextColor), true, None);
    let bg_color_item = MenuItem::with_id("bg_color", i18n::tr(i18n::TrKey::BgColor), true, None);
    let opacity_item = MenuItem::with_id("opacity", i18n::tr(i18n::TrKey::Opacity), true, None);
    let restart_item = MenuItem::with_id("restart", i18n::tr(i18n::TrKey::Restart), true, None);
    let exit_item = MenuItem::with_id("exit", i18n::tr(i18n::TrKey::Exit), true, None);
    let sep = PredefinedMenuItem::separator();
    let sep2 = PredefinedMenuItem::separator();
    let sep3 = PredefinedMenuItem::separator();
    let sep4 = PredefinedMenuItem::separator();
    let sep5 = PredefinedMenuItem::separator();

    MenuEvent::set_event_handler(Some(Box::new(|event: MenuEvent| {
        debug_log(format!("[main] tray menu clicked: {:?}\n", event.id));
        match event.id.as_ref() {
            "exit" => {
                debug_log("[main] tray menu: exit\n");
                quit_app(0);
            }
            "restart" => {
                debug_log("[main] tray menu: restart\n");
                restart_app();
            }
            "skip_next" => {
                debug_log("[main] tray menu: skip next\n");
                let count = SKIP_COUNT.get().unwrap();
                let _ = count.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    Some(value.saturating_add(1))
                });
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
                if let Err(e) = std::process::Command::new("notepad.exe").arg(&path).spawn() {
                    report_error("Failed to open config in Notepad", e);
                }
            }
            "text_color" => {
                debug_log("[main] tray menu: text color\n");
                dialog_choose_color(ColorKind::Text);
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "bg_color" => {
                debug_log("[main] tray menu: background color\n");
                dialog_choose_color(ColorKind::Bg);
                NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
            }
            "opacity" => {
                debug_log("[main] tray menu: opacity\n");
                gui::toggle_opacity_panel();
            }
            _ => {
                debug_log(format!("[main] tray menu: unknown id '{:?}'\n", event.id));
            }
        }
    })));

    let menu = Menu::new();
    for item in [
        &next_item as &dyn tray_icon::menu::IsMenuItem,
        &sep,
        &show_item,
        &skip_item,
        &pause_item,
        &sep2,
        &edit_item,
        &sep3,
        &color_item,
        &bg_color_item,
        &opacity_item,
        &sep4,
        &restart_item,
        &sep5,
        &exit_item,
    ] {
        menu.append(item)
            .unwrap_or_else(|e| fatal(&format!("Failed to build tray menu: {e}")));
    }

    // Create tray icon from embedded 256×256 RGBA raw data
    let icon_raw: &[u8] = include_bytes!("../res/icon_raw");
    let icon =
        tray_icon::Icon::from_rgba(icon_raw.to_vec(), 256, 256).expect("tray icon from raw RGBA");

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

    let mut last_checked_second: Option<u32> = None;
    let mut last_menu_refresh = std::time::Instant::now();
    let mut last_config_check = std::time::Instant::now();
    let mut last_reload_error: Option<String> = None;

    // Initial menu refresh
    refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);

    debug_log("[main] entering main loop\n");
    let mut loop_count: u64 = 0;

    loop {
        loop_count += 1;
        if loop_count % 120 == 1 {
            // Heartbeat roughly every 60 seconds (120 iterations × 500ms)
            debug_log(format!("[main] heartbeat: iteration {loop_count}\n"));
        }
        pump_messages();

        // WM_HOTKEY is dispatched to the clock window by the message pump.
        let now = Local::now();
        let current = (now.hour(), now.minute(), now.second());
        let current_second = current.0 * 3600 + current.1 * 60 + current.2;

        // Limited hot reload: only display_time and schedule/audio are accepted
        // from external file edits. Other runtime-owned fields remain unchanged.
        if last_config_check.elapsed() >= Duration::from_secs(1) {
            last_config_check = std::time::Instant::now();
            let reload_result = {
                let mut cfg = CONFIG.get().unwrap().lock().unwrap();
                match cfg.reload_hot_fields() {
                    Ok(changed) => {
                        if changed {
                            gui::update_config(&cfg.general);
                        }
                        Ok(changed)
                    }
                    Err(error) => Err(error),
                }
            };
            match reload_result {
                Ok(changed) => {
                    if changed {
                        debug_log("[main] config hot reload applied\n");
                        NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
                    }
                    last_reload_error = None;
                }
                Err(error) => {
                    if last_reload_error.as_deref() != Some(&error) {
                        debug_log(format!("[main] config hot reload ignored: {error}\n"));
                        last_reload_error = Some(error);
                    }
                }
            }
        }

        // Menu state refresh (immediate if needed, otherwise periodic)
        if NEED_REFRESH.get().unwrap().swap(false, Ordering::Relaxed) {
            refresh_menu_items(&tray, &next_item, &pause_item, &skip_item, &show_item);
            last_menu_refresh = std::time::Instant::now();
        }

        // Schedule check: every second
        if last_checked_second != Some(current_second) {
            // Do not replay the entire day on startup or after midnight. For a
            // normal forward tick, include all seconds crossed since the last
            // iteration so brief UI stalls do not lose reminders.
            let range_start = match last_checked_second {
                Some(previous) if previous < current_second => previous,
                _ => current_second.saturating_sub(1),
            };
            last_checked_second = Some(current_second);

            let paused = PAUSED.get().unwrap().load(Ordering::Relaxed);

            // Scope the config lock so it's released before refresh_menu_items (avoids deadlock)
            let matches_found = {
                let cfg = CONFIG.get().unwrap().lock().unwrap();
                let entries = cfg.entries_between(range_start, current_second);

                // Determine if we should skip the next match
                let do_skip = if paused {
                    true
                } else if !entries.is_empty() {
                    let count = SKIP_COUNT.get().unwrap();
                    count
                        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                            if v > 0 { Some(v - 1) } else { None }
                        })
                        .is_ok()
                } else {
                    false
                };

                if !do_skip && !entries.is_empty() {
                    for entry in &entries {
                        debug_log(format!(
                            "[main] schedule match at {:02}:{:02}:{:02}, audio={:?}\n",
                            current.0, current.1, current.2, entry.audio
                        ));
                        if let Some(audio) = entry.audio.as_deref() {
                            AUDIO.get().unwrap().play(audio, &cfg.config_dir);
                        }
                    }
                    // Omitting audio means a silent visual reminder.
                    gui::show_clock();
                    true
                } else if do_skip && !entries.is_empty() {
                    // A scheduled entry was skipped — update the menu/tooltip so the
                    // user immediately sees the *next* reminder after the skipped one.
                    NEED_REFRESH.get().unwrap().store(true, Ordering::Relaxed);
                    false
                } else {
                    false
                }
            }; // cfg lock released here

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

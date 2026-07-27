#![allow(clippy::upper_case_acronyms)]

use crate::config::GeneralConfig;
use chrono::{Local, Timelike};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

// ───────────────────────────────────────────────
//  Win32 FFI declarations
// ───────────────────────────────────────────────

type HINSTANCE = *mut std::ffi::c_void;
type HWND = *mut std::ffi::c_void;
type HDC = *mut std::ffi::c_void;
type HGDIOBJ = *mut std::ffi::c_void;
type HBRUSH = *mut std::ffi::c_void;
type HFONT = *mut std::ffi::c_void;
type HBITMAP = *mut std::ffi::c_void;
type LPARAM = isize;
type WPARAM = usize;
type LRESULT = isize;
type ATOM = u16;

const WS_POPUP: u32 = 0x8000_0000;
const WS_EX_LAYERED: u32 = 0x0008_0000;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_TOPMOST: u32 = 0x0000_0008;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

const ULW_ALPHA: u32 = 0x0000_0002;
const AC_SRC_ALPHA: u8 = 0x01;

const BI_RGB: u32 = 0;

const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;

const SWP_NOSIZE: u32 = 0x0001;
#[allow(dead_code)]
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOACTIVATE: u32 = 0x0010;
#[allow(dead_code)]
const SWP_SHOWWINDOW: u32 = 0x0040;
const SW_HIDE: i32 = 0;

const AW_HOR_POSITIVE: u32 = 0x0000_0001;
const AW_HOR_NEGATIVE: u32 = 0x0000_0002;
const AW_SLIDE: u32 = 0x0004_0000;
const AW_HIDE: u32 = 0x0001_0000;

const TRANSPARENT: i32 = 1;

const WM_HOTKEY: u32 = 0x0312;
const WM_USER_HOTKEY: u32 = 0x0401; // custom: posted by hotkey window
const WM_NULL: u32 = 0x0000;
const WM_TIMER: u32 = 0x0113;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_COMMAND: u32 = 0x0111;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const HTCAPTION: isize = 2;

// Popup menu flags
const MF_STRING: u32 = 0x0000_0000;
const MF_SEPARATOR: u32 = 0x0000_0800;
const TPM_RIGHTBUTTON: u32 = 0x0000_0002;
const TPM_NONOTIFY: u32 = 0x0000_0080;

// Menu command IDs
const IDM_HIDE_CLOCK: usize = 1001;
const IDM_EXIT: usize = 1002;

const FW_NORMAL: i32 = 400;
const DEFAULT_CHARSET: u8 = 1;
const OUT_DEFAULT_PRECIS: u8 = 0;
const CLIP_DEFAULT_PRECIS: u8 = 0;
const DEFAULT_QUALITY: u8 = 0;
const DEFAULT_PITCH: u32 = 0;
const FF_DONTCARE: u32 = 0;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct BITMAPINFOHEADER {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[repr(C)]
struct BLENDFUNCTION {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HINSTANCE,
    h_cursor: HINSTANCE,
    hbr_background: HBRUSH,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HINSTANCE,
}

#[link(name = "user32")]
unsafe extern "system" {
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
        hMenu: HINSTANCE,
        hInstance: HINSTANCE,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;

    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> ATOM;
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn GetDC(hWnd: HWND) -> HDC;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
    fn CreateCompatibleDC(hDC: HDC) -> HDC;
    fn DeleteDC(hDC: HDC) -> i32;
    fn CreateDIBSection(
        hdc: HDC,
        pbmi: *const BITMAPINFOHEADER,
        usage: u32,
        ppvBits: *mut *mut std::ffi::c_void,
        hSection: HINSTANCE,
        offset: u32,
    ) -> HBITMAP;
    fn SelectObject(hDC: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn DeleteObject(ho: HGDIOBJ) -> i32;
    fn UpdateLayeredWindow(
        hWnd: HWND,
        hdcDst: HDC,
        pptDst: *const POINT,
        psize: *const i32,
        hdcSrc: HDC,
        pptSrc: *const POINT,
        crKey: u32,
        pblend: *const BLENDFUNCTION,
        dwFlags: u32,
    ) -> i32;
    fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn AnimateWindow(hWnd: HWND, dwTime: u32, dwFlags: u32) -> i32;
    fn SetTimer(
        hWnd: HWND,
        nIDEvent: usize,
        uElapse: u32,
        lpTimerFunc: *mut std::ffi::c_void,
    ) -> usize;
    fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;
    fn CreateFontW(
        nHeight: i32,
        nWidth: i32,
        nEscapement: i32,
        nOrientation: i32,
        fnWeight: i32,
        fdwItalic: u32,
        fdwUnderline: u32,
        fdwStrikeOut: u32,
        fdwCharSet: u32,
        fdwOutputPrecision: u32,
        fdwClipPrecision: u32,
        fdwQuality: u32,
        fdwPitchAndFamily: u32,
        lpszFace: *const u16,
    ) -> HFONT;
    fn TextOutW(hdc: HDC, x: i32, y: i32, lpString: *const u16, c: i32) -> i32;
    fn GetTextExtentPoint32W(hdc: HDC, lpString: *const u16, c: i32, lpSizel: *mut i32) -> i32;
    fn SetTextColor(hdc: HDC, color: u32) -> u32;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ReleaseCapture() -> i32;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    fn CreatePopupMenu() -> *mut std::ffi::c_void;
    fn AppendMenuW(
        hMenu: *mut std::ffi::c_void,
        uFlags: u32,
        uIDNewItem: usize,
        lpNewItem: *const u16,
    ) -> i32;
    fn TrackPopupMenu(
        hMenu: *mut std::ffi::c_void,
        uFlags: u32,
        x: i32,
        y: i32,
        nReserved: i32,
        hWnd: HWND,
        prcRect: *const std::ffi::c_void,
    ) -> i32;
    fn DestroyMenu(hMenu: *mut std::ffi::c_void) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn SetForegroundWindow(hWnd: HWND) -> i32;
    fn PostMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

// ───────────────────────────────────────────────
//  Raw pointer wrapper (Send + Sync, main thread only)
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct RawPtr(isize);

unsafe impl Send for RawPtr {}
unsafe impl Sync for RawPtr {}

impl RawPtr {
    fn from_ptr<T>(p: *mut T) -> Self {
        RawPtr(p as isize)
    }
    fn as_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
    fn as_hwnd(&self) -> HWND {
        self.0 as HWND
    }
    fn as_hdc(&self) -> HDC {
        self.0 as HDC
    }
    fn as_hgdiobj(&self) -> HGDIOBJ {
        self.0 as HGDIOBJ
    }
}

// ───────────────────────────────────────────────
//  Helpers
// ───────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ───────────────────────────────────────────────
//  Clock window state
// ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClockWindowConfig {
    #[allow(dead_code)]
    pub font_name: String,
    #[allow(dead_code)]
    pub font_size: i32,
    pub bg_color: (u8, u8, u8, u8), // r, g, b, opacity(0-100)
    pub text_color: (u8, u8, u8),
    pub display_time: u32,
}

struct GuiState {
    hwnd: RawPtr,
    config: ClockWindowConfig,
    shown_at: Option<std::time::Instant>,
    width: i32,
    height: i32,
    slide_left: bool,
    // Bitmap resources
    mem_dc: RawPtr,
    bitmap: RawPtr,
    bitmap_bits: RawPtr, // *mut u8
    font: RawPtr,
    last_time_str: String,
    timer_update_id: usize,
}

static GUI_STATE: OnceLock<Mutex<Option<GuiState>>> = OnceLock::new();
static GUI_VISIBLE: AtomicBool = AtomicBool::new(false);
static GUI_HWND_STATIC: AtomicI32 = AtomicI32::new(0);

// ───────────────────────────────────────────────
//  Window procedure
// ───────────────────────────────────────────────

unsafe extern "system" fn clock_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY | WM_USER_HOTKEY => {
            if is_visible() {
                hide_clock();
            } else {
                show_clock();
            }
            return 0;
        }

        WM_TIMER => {
            let timer_id = wparam;
            if let Some(state_lock) = GUI_STATE.get() {
                let mut state_opt = state_lock.lock().unwrap();
                if let Some(ref mut state) = *state_opt
                    && timer_id == state.timer_update_id
                {
                    let now = Local::now();
                    let time_str =
                        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
                    if time_str != state.last_time_str {
                        state.last_time_str = time_str;
                        unsafe {
                            redraw_layered_window(state);
                        }
                    }

                    if let Some(shown_at) = state.shown_at {
                        let elapsed = shown_at.elapsed().as_secs() as u32;
                        if elapsed >= state.config.display_time {
                            unsafe {
                                hide_clock_internal(state);
                            }
                        }
                    }
                }
            }
            return 0;
        }

        WM_LBUTTONDOWN => {
            unsafe {
                ReleaseCapture();
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            }
            if let Some(state_lock) = GUI_STATE.get() {
                let mut state_opt = state_lock.lock().unwrap();
                if let Some(ref mut state) = *state_opt {
                    state.shown_at = Some(std::time::Instant::now());
                }
            }
            return 0;
        }

        WM_RBUTTONUP => {
            // Show a popup context menu on the clock window
            unsafe {
                show_clock_context_menu(hwnd, lparam);
            }
            return 0;
        }

        WM_COMMAND => {
            match wparam {
                IDM_HIDE_CLOCK => hide_clock(),
                IDM_EXIT => std::process::exit(0),
                _ => {}
            }
            return 0;
        }

        WM_DESTROY | WM_CLOSE => {
            hide_clock();
            return 0;
        }

        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Show a right-click popup menu on the clock window.
unsafe fn show_clock_context_menu(hwnd: HWND, _lparam: LPARAM) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        // "Hide Clock" item
        let hide_text = crate::i18n::tr(crate::i18n::TrKey::ShowClock); // reuse key
        let hide_wide: Vec<u16> = hide_text.encode_utf16().chain(std::iter::once(0)).collect();
        AppendMenuW(menu, MF_STRING, IDM_HIDE_CLOCK, hide_wide.as_ptr());

        // Separator
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());

        // "Exit" item
        let exit_text = crate::i18n::tr(crate::i18n::TrKey::Exit);
        let exit_wide: Vec<u16> = exit_text.encode_utf16().chain(std::iter::once(0)).collect();
        AppendMenuW(menu, MF_STRING, IDM_EXIT, exit_wide.as_ptr());

        // Get cursor position
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);

        // Must set foreground window so the menu dismisses properly
        SetForegroundWindow(hwnd);

        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_NONOTIFY,
            pt.x,
            pt.y,
            0,
            hwnd,
            std::ptr::null(),
        );

        // Post a dummy message to make the menu dismiss on next click
        PostMessageW(hwnd, WM_NULL, 0, 0);

        DestroyMenu(menu);
    }
}

// ───────────────────────────────────────────────
//  Redraw
// ───────────────────────────────────────────────

unsafe fn redraw_layered_window(state: &mut GuiState) {
    unsafe {
        let w = state.width as isize;
        let h = state.height as isize;
        let hdc = state.mem_dc.as_hdc();
        let bits_ptr = state.bitmap_bits.as_ptr::<u8>();

        let (br, bg, bb, alpha_pct) = state.config.bg_color;
        let alpha = ((alpha_pct as f32 / 100.0) * 255.0) as u8;

        // Fill background (BGRA)
        {
            let bits = std::slice::from_raw_parts_mut(bits_ptr, (w * h * 4) as usize);
            for y in 0..h {
                for x in 0..w {
                    let idx = ((y * w + x) * 4) as usize;
                    bits[idx] = bg;
                    bits[idx + 1] = br;
                    bits[idx + 2] = bb;
                    bits[idx + 3] = alpha;
                }
            }
        }

        // Draw text
        let old_font = SelectObject(hdc, state.font.as_hgdiobj());
        let (tr, tg, tb) = state.config.text_color;
        let text_rgb: u32 = (tr as u32) | ((tg as u32) << 8) | ((tb as u32) << 16);
        SetTextColor(hdc, text_rgb);
        SetBkMode(hdc, TRANSPARENT);

        let time_wide = to_wide(&state.last_time_str);
        let mut extent = [0i32; 2];
        GetTextExtentPoint32W(
            hdc,
            time_wide.as_ptr(),
            (time_wide.len() - 1) as i32,
            extent.as_mut_ptr(),
        );
        let text_w = extent[0];
        let text_h = extent[1];

        let x = (w - text_w as isize) / 2;
        let y = (h - text_h as isize) / 2;
        TextOutW(
            hdc,
            x as i32,
            y as i32,
            time_wide.as_ptr(),
            (time_wide.len() - 1) as i32,
        );

        SelectObject(hdc, old_font);

        // Post-process: set alpha=255 for text pixels
        {
            let bits = std::slice::from_raw_parts_mut(bits_ptr, (w * h * 4) as usize);
            for y in 0..h {
                for x in 0..w {
                    let idx = ((y * w + x) * 4) as usize;
                    let b = bits[idx];
                    let g = bits[idx + 1];
                    let r = bits[idx + 2];
                    if b != bg || g != br || r != bb {
                        bits[idx + 3] = 255;
                    }
                }
            }
        }

        // Update
        let blend = BLENDFUNCTION {
            blend_op: 0,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };
        let pt_src = POINT { x: 0, y: 0 };
        let size = [state.width, state.height];
        let screen_dc = GetDC(std::ptr::null_mut());
        UpdateLayeredWindow(
            state.hwnd.as_hwnd(),
            screen_dc,
            std::ptr::null(),
            size.as_ptr(),
            hdc,
            &pt_src,
            0,
            &blend,
            ULW_ALPHA,
        );
        ReleaseDC(std::ptr::null_mut(), screen_dc);
    }
}

// ───────────────────────────────────────────────
//  Show / hide
// ───────────────────────────────────────────────

unsafe fn hide_clock_internal(state: &mut GuiState) {
    unsafe {
        let hwnd = state.hwnd.as_hwnd();
        KillTimer(hwnd, state.timer_update_id);
        state.shown_at = None;
        GUI_VISIBLE.store(false, Ordering::Relaxed);

        if state.slide_left {
            AnimateWindow(hwnd, 400, AW_HIDE | AW_SLIDE | AW_HOR_NEGATIVE);
        } else {
            AnimateWindow(hwnd, 400, AW_HIDE | AW_SLIDE | AW_HOR_POSITIVE);
        }
        ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn hide_clock() {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref mut state) = *state_opt {
            unsafe {
                hide_clock_internal(state);
            }
        }
    }
}

pub fn show_clock() {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref mut state) = *state_opt {
            unsafe {
                show_clock_internal(state);
            }
        }
    }
}

unsafe fn show_clock_internal(state: &mut GuiState) {
    unsafe {
        let hwnd = state.hwnd.as_hwnd();

        if GUI_VISIBLE.load(Ordering::Relaxed) {
            state.shown_at = Some(std::time::Instant::now());
            return;
        }

        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let x = (sw - state.width) / 2;
        let y = sh / 6;

        state.slide_left = x < sw / 2;

        // Hide and position off-screen as animation start point
        ShowWindow(hwnd, SW_HIDE);

        let start_x = if state.slide_left { -(state.width) } else { sw };

        SetWindowPos(
            hwnd,
            (-1isize) as HWND,
            start_x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );

        // Render the current time onto the bitmap
        let now = Local::now();
        state.last_time_str = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
        redraw_layered_window(state);

        // Animate in (this also shows the window)
        if state.slide_left {
            AnimateWindow(hwnd, 600, AW_SLIDE | AW_HOR_POSITIVE);
        } else {
            AnimateWindow(hwnd, 600, AW_SLIDE | AW_HOR_NEGATIVE);
        }

        // Ensure final position and topmost
        SetWindowPos(
            hwnd,
            (-1isize) as HWND,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );

        GUI_VISIBLE.store(true, Ordering::Relaxed);
        state.shown_at = Some(std::time::Instant::now());

        KillTimer(hwnd, state.timer_update_id);
        SetTimer(hwnd, state.timer_update_id, 500, std::ptr::null_mut());
    }
}

// ───────────────────────────────────────────────
//  Create
// ───────────────────────────────────────────────

pub fn create_clock_window(cfg: &GeneralConfig) -> Result<(), String> {
    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    if hinst.is_null() {
        return Err("获取模块句柄失败".into());
    }

    let class_name = to_wide("TipClockWindowClass");
    let wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT = clock_wndproc;

    let wc = WNDCLASSEXW {
        cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfn_wnd_proc: wnd_proc,
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
        return Err("注册窗口类失败".into());
    }

    // Measure text using font from config
    let font_size = -cfg.font_size; // negative = logical point size for CreateFontW
    let font_name = cfg.font_name.as_str();
    let display_str = "00:00:00";

    let hdc = unsafe { GetDC(std::ptr::null_mut()) };
    let font_wide = to_wide(font_name);
    let font = unsafe {
        CreateFontW(
            font_size,
            0,
            0,
            0,
            FW_NORMAL,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            DEFAULT_PITCH | FF_DONTCARE,
            font_wide.as_ptr(),
        )
    };
    let old_font = unsafe { SelectObject(hdc, font as HGDIOBJ) };

    let wide_display = to_wide(display_str);
    let mut extent = [0i32; 2];
    unsafe {
        GetTextExtentPoint32W(
            hdc,
            wide_display.as_ptr(),
            (wide_display.len() - 1) as i32,
            extent.as_mut_ptr(),
        );
    }

    let text_w = extent[0];
    let text_h = extent[1];
    let pad_x: i32 = 20;
    let pad_y: i32 = 10;
    let win_w = text_w + pad_x * 2;
    let win_h = text_h + pad_y * 2;

    unsafe {
        SelectObject(hdc, old_font);
        DeleteObject(font as HGDIOBJ);
        ReleaseDC(std::ptr::null_mut(), hdc);
    }

    // Create window
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            std::ptr::null(),
            WS_POPUP,
            0,
            0,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        return Err("创建窗口失败".into());
    }

    // Resources for layered rendering
    let mem_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };

    let bmi = BITMAPINFOHEADER {
        bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        bi_width: win_w,
        bi_height: win_h,
        bi_planes: 1,
        bi_bit_count: 32,
        bi_compression: BI_RGB,
        bi_size_image: 0,
        bi_x_pels_per_meter: 0,
        bi_y_pels_per_meter: 0,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };

    let mut bitmap_bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let bitmap = unsafe {
        CreateDIBSection(
            mem_dc,
            &bmi,
            0,
            &mut bitmap_bits as *mut _,
            std::ptr::null_mut(),
            0,
        )
    };
    unsafe {
        SelectObject(mem_dc, bitmap as HGDIOBJ);
    }

    let font = unsafe {
        CreateFontW(
            font_size,
            0,
            0,
            0,
            FW_NORMAL,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            DEFAULT_PITCH | FF_DONTCARE,
            font_wide.as_ptr(),
        )
    };

    let window_config = ClockWindowConfig {
        font_name: font_name.to_string(),
        font_size,
        bg_color: (cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_opacity),
        text_color: (cfg.text_r, cfg.text_g, cfg.text_b),
        display_time: cfg.display_time,
    };

    let state = GuiState {
        hwnd: RawPtr::from_ptr(hwnd),
        config: window_config,
        shown_at: None,
        width: win_w,
        height: win_h,
        slide_left: true,
        mem_dc: RawPtr::from_ptr(mem_dc),
        bitmap: RawPtr::from_ptr(bitmap),
        bitmap_bits: RawPtr::from_ptr(bitmap_bits),
        font: RawPtr::from_ptr(font),
        last_time_str: String::new(),
        timer_update_id: 1,
    };

    GUI_STATE.set(Mutex::new(Some(state))).ok();
    GUI_HWND_STATIC.store(hwnd as i32, Ordering::Relaxed);

    Ok(())
}

pub fn get_hwnd() -> HWND {
    GUI_HWND_STATIC.load(Ordering::Relaxed) as HWND
}

pub fn is_visible() -> bool {
    GUI_VISIBLE.load(Ordering::Relaxed)
}

pub fn update_config(cfg: &GeneralConfig) {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref mut state) = *state_opt {
            state.config.bg_color = (cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_opacity);
            state.config.text_color = (cfg.text_r, cfg.text_g, cfg.text_b);
            state.config.display_time = cfg.display_time;
            if GUI_VISIBLE.load(Ordering::Relaxed) {
                unsafe {
                    redraw_layered_window(state);
                }
            }
        }
    }
}

/// Update font (name/size) and optionally re-create the window resources.
/// This is called after the user picks a new font via the system dialog.
pub fn update_font(cfg: &GeneralConfig) {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref mut state) = *state_opt {
            unsafe {
                // Delete old font
                DeleteObject(state.font.as_hgdiobj());

                // Create new font
                let font_size = -cfg.font_size; // negative = logical point size
                let font_wide = to_wide(&cfg.font_name);
                let new_font = CreateFontW(
                    font_size,
                    0,
                    0,
                    0,
                    FW_NORMAL,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET as u32,
                    OUT_DEFAULT_PRECIS as u32,
                    CLIP_DEFAULT_PRECIS as u32,
                    DEFAULT_QUALITY as u32,
                    DEFAULT_PITCH | FF_DONTCARE,
                    font_wide.as_ptr(),
                );

                // Measure new text extent to resize window
                let hdc = GetDC(std::ptr::null_mut());
                let old_f = SelectObject(hdc, new_font as HGDIOBJ);
                let display_wide = to_wide("00:00:00");
                let mut extent = [0i32; 2];
                GetTextExtentPoint32W(
                    hdc,
                    display_wide.as_ptr(),
                    (display_wide.len() - 1) as i32,
                    extent.as_mut_ptr(),
                );
                SelectObject(hdc, old_f);
                ReleaseDC(std::ptr::null_mut(), hdc);

                let pad_x: i32 = 20;
                let pad_y: i32 = 10;
                let new_w = extent[0] + pad_x * 2;
                let new_h = extent[1] + pad_y * 2;

                // Resize the DIB section if needed
                if new_w != state.width || new_h != state.height {
                    DeleteObject(state.bitmap.as_hgdiobj());
                    let bmi = BITMAPINFOHEADER {
                        bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        bi_width: new_w,
                        bi_height: new_h,
                        bi_planes: 1,
                        bi_bit_count: 32,
                        bi_compression: BI_RGB,
                        bi_size_image: 0,
                        bi_x_pels_per_meter: 0,
                        bi_y_pels_per_meter: 0,
                        bi_clr_used: 0,
                        bi_clr_important: 0,
                    };
                    let mut new_bits: *mut std::ffi::c_void = std::ptr::null_mut();
                    let new_bmp = CreateDIBSection(
                        state.mem_dc.as_hdc(),
                        &bmi,
                        0,
                        &mut new_bits as *mut _,
                        std::ptr::null_mut(),
                        0,
                    );
                    SelectObject(state.mem_dc.as_hdc(), new_bmp as HGDIOBJ);
                    state.bitmap = RawPtr::from_ptr(new_bmp);
                    state.bitmap_bits = RawPtr::from_ptr(new_bits);
                    state.width = new_w;
                    state.height = new_h;
                }

                state.font = RawPtr::from_ptr(new_font);
                state.config.font_name = cfg.font_name.clone();
                state.config.font_size = cfg.font_size;

                if GUI_VISIBLE.load(Ordering::Relaxed) {
                    redraw_layered_window(state);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn destroy_clock_window() {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref state) = *state_opt {
            unsafe {
                KillTimer(state.hwnd.as_hwnd(), state.timer_update_id);
                DeleteObject(state.font.as_hgdiobj());
                DeleteObject(state.bitmap.as_hgdiobj());
                DeleteDC(state.mem_dc.as_hdc());
                DestroyWindow(state.hwnd.as_hwnd());
            }
        }
        *state_opt = None;
    }
}

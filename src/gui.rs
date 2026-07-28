#![allow(clippy::upper_case_acronyms)]

use crate::config::GeneralConfig;
use chrono::{Local, Timelike};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

// ───────────────────────────────────────────────
//  GDI+ types & constants
// ───────────────────────────────────────────────

type GpGraphics = *mut std::ffi::c_void;
type GpBrush = *mut std::ffi::c_void;
type GpFontFamily = *mut std::ffi::c_void;
type GpFont = *mut std::ffi::c_void;
type GpStringFormat = *mut std::ffi::c_void;
#[allow(non_camel_case_types)]
type ULONG_PTR = usize;

const GDI_PLUS_OK: i32 = 0;
const FONT_STYLE_REGULAR: i32 = 0;
const UNIT_POINT: i32 = 3; // 1/72 inch
const STRING_ALIGN_CENTER: i32 = 1;
const TEXT_RENDERING_HINT_ANTIALIAS: i32 = 4;
const SMOOTHING_MODE_HIGH_QUALITY: i32 = 2;

#[repr(C)]
struct GdiplusStartupInputStruct {
    gdiplus_version: u32,
    debug_event_callback: *mut std::ffi::c_void,
    suppress_background_thread: i32,
    suppress_external_codecs: i32,
}

impl Default for GdiplusStartupInputStruct {
    fn default() -> Self {
        GdiplusStartupInputStruct {
            gdiplus_version: 1,
            debug_event_callback: std::ptr::null_mut(),
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        }
    }
}

#[repr(C)]
struct RectF {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

// ───────────────────────────────────────────────
//  GDI+ FFI (gdiplus.dll)
// ───────────────────────────────────────────────

#[link(name = "gdiplus")]
unsafe extern "system" {
    fn GdiplusStartup(
        token: *mut ULONG_PTR,
        input: *const GdiplusStartupInputStruct,
        output: *mut std::ffi::c_void,
    ) -> i32;
    fn GdiplusShutdown(token: ULONG_PTR);
    fn GdipCreateFromHDC(hdc: HDC, graphics: *mut GpGraphics) -> i32;
    fn GdipDeleteGraphics(graphics: GpGraphics) -> i32;
    fn GdipCreateSolidFill(color: u32, brush: *mut GpBrush) -> i32;
    fn GdipDeleteBrush(brush: GpBrush) -> i32;
    fn GdipFillRectangleI(
        graphics: GpGraphics,
        brush: GpBrush,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> i32;
    fn GdipCreateFontFamilyFromName(
        name: *const u16,
        font_collection: *mut std::ffi::c_void,
        font_family: *mut GpFontFamily,
    ) -> i32;
    fn GdipDeleteFontFamily(font_family: GpFontFamily) -> i32;
    fn GdipCreateFont(
        font_family: GpFontFamily,
        em_size: f32,
        style: i32,
        unit: i32,
        font: *mut GpFont,
    ) -> i32;
    fn GdipDeleteFont(font: GpFont) -> i32;
    fn GdipCreateStringFormat(
        format_attributes: i32,
        language: u16,
        format: *mut GpStringFormat,
    ) -> i32;
    fn GdipDeleteStringFormat(format: GpStringFormat) -> i32;
    fn GdipSetStringFormatAlign(format: GpStringFormat, align: i32) -> i32;
    fn GdipSetStringFormatLineAlign(format: GpStringFormat, align: i32) -> i32;
    fn GdipDrawString(
        graphics: GpGraphics,
        string: *const u16,
        length: i32,
        font: GpFont,
        layout_rect: *const RectF,
        string_format: GpStringFormat,
        brush: GpBrush,
    ) -> i32;
    fn GdipSetTextRenderingHint(graphics: GpGraphics, mode: i32) -> i32;
    fn GdipSetSmoothingMode(graphics: GpGraphics, mode: i32) -> i32;
    fn GdipGetFontHeight(font: GpFont, graphics: GpGraphics, height: *mut f32) -> i32;
}

// ───────────────────────────────────────────────
//  Win32 types & constants (window management)
// ───────────────────────────────────────────────

type HINSTANCE = *mut std::ffi::c_void;
type HWND = *mut std::ffi::c_void;
type HDC = *mut std::ffi::c_void;
type HGDIOBJ = *mut std::ffi::c_void;
type HBRUSH = *mut std::ffi::c_void;
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

const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SW_HIDE: i32 = 0;

const WM_HOTKEY: u32 = 0x0312;
const WM_USER_HOTKEY: u32 = 0x0401;
const WM_NULL: u32 = 0x0000;
const WM_TIMER: u32 = 0x0113;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_COMMAND: u32 = 0x0111;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_SETCURSOR: u32 = 0x0020;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const HTCAPTION: isize = 2;

const IDC_ARROW: *const u16 = 32512usize as *const u16;

const LOGPIXELSY: i32 = 90; // pixels per logical inch (vertical DPI)

const MF_STRING: u32 = 0x0000_0000;
const MF_SEPARATOR: u32 = 0x0000_0800;
const TPM_RIGHTBUTTON: u32 = 0x0000_0002;
const TPM_NONOTIFY: u32 = 0x0000_0080;

const IDM_HIDE_CLOCK: usize = 1001;
const IDM_EXIT: usize = 1002;

// ───────────────────────────────────────────────
//  Win32 structures
// ───────────────────────────────────────────────

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
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

// ───────────────────────────────────────────────
//  Win32 FFI
// ───────────────────────────────────────────────

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
    fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HINSTANCE;
    fn SetCursor(hCursor: HINSTANCE) -> HINSTANCE;
    fn SetTimer(
        hWnd: HWND,
        nIDEvent: usize,
        uElapse: u32,
        lpTimerFunc: *mut std::ffi::c_void,
    ) -> usize;
    fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;
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
    fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
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

/// GDI+ object wrapper — all GDI+ objects are thread-safe when used on the
/// main thread exclusively (which we guarantee).
#[derive(Debug, Clone, Copy)]
struct GpObj(isize);
unsafe impl Send for GpObj {}
unsafe impl Sync for GpObj {}

impl GpObj {
    fn gp_font_family(&self) -> GpFontFamily {
        self.0 as GpFontFamily
    }
    fn gp_font(&self) -> GpFont {
        self.0 as GpFont
    }
    fn gp_string_format(&self) -> GpStringFormat {
        self.0 as GpStringFormat
    }
}

// ───────────────────────────────────────────────
//  Helpers
// ───────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn debug_log(s: impl ToString) {
    crate::audio::debug_log(s);
}

/// Ease-out cubic: fast start, smooth deceleration.
fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u
}

fn get_system_dpi() -> f32 {
    unsafe {
        let screen_dc = GetDC(std::ptr::null_mut());
        let dpi = GetDeviceCaps(screen_dc, LOGPIXELSY) as f32;
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        dpi
    }
}

// ───────────────────────────────────────────────
//  GDI+ one-time initialisation
// ───────────────────────────────────────────────

static GDI_PLUS_TOKEN: OnceLock<ULONG_PTR> = OnceLock::new();

fn gdiplus_init() -> Result<(), String> {
    let input = GdiplusStartupInputStruct::default();
    let mut token: ULONG_PTR = 0;
    let status = unsafe { GdiplusStartup(&mut token, &input, std::ptr::null_mut()) };
    if status != GDI_PLUS_OK {
        return Err(format!("GdiplusStartup failed (status={status})"));
    }
    GDI_PLUS_TOKEN.set(token).ok();
    Ok(())
}

fn gdiplus_shutdown() {
    if let Some(token) = GDI_PLUS_TOKEN.get() {
        unsafe { GdiplusShutdown(*token) };
    }
}

// Hardcoded font parameters — no longer read from config.
const FONT_NAME: &str = "Microsoft YaHei UI";
const FONT_SIZE_PT: f32 = 18.0;
const DISPLAY_STR: &str = "88:88:88";
const PAD_X: i32 = 6;
const PAD_Y: i32 = 6;

// ───────────────────────────────────────────────
//  Clock window state
// ───────────────────────────────────────────────

const ANIM_TIMER_ID: usize = 2;
const ANIM_INTERVAL_MS: u32 = 16;
const SLIDE_IN_MS: u32 = 1000;
const SLIDE_OUT_MS: u32 = 3000;
const SLIDE_DISTANCE: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimKind {
    Enter,
    Exit,
}

struct Animation {
    kind: AnimKind,
    start: std::time::Instant,
    start_y: i32,
    end_y: i32,
}

#[derive(Debug, Clone)]
pub struct ClockWindowConfig {
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
    text_h: i32,
    // GDI objects
    mem_dc: RawPtr,
    bitmap: RawPtr,
    bitmap_bits: RawPtr,
    // GDI+ objects
    gp_font_family: GpObj,
    gp_font: GpObj,
    gp_string_format: GpObj,
    last_time_str: String,
    timer_update_id: usize,
    animation: Option<Animation>,
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
            debug_log("[gui] WM_HOTKEY / WM_USER_HOTKEY received\n");
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
                if let Some(ref mut state) = *state_opt {
                    if timer_id == ANIM_TIMER_ID {
                        unsafe {
                            process_animation_frame(state);
                        }
                        if state.animation.is_none() && !GUI_VISIBLE.load(Ordering::Relaxed) {
                            unsafe {
                                KillTimer(hwnd, ANIM_TIMER_ID);
                            }
                        }
                    } else if timer_id == state.timer_update_id {
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
            }
            return 0;
        }

        WM_LBUTTONDOWN => {
            unsafe {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                ReleaseCapture();
                let lparam = ((pt.y as u32) << 16) as isize | ((pt.x as u32) & 0xFFFF) as isize;
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, lparam);
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
            unsafe {
                show_clock_context_menu(hwnd);
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

        WM_SETCURSOR => {
            unsafe {
                let arrow = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
                if !arrow.is_null() {
                    SetCursor(arrow);
                }
            }
            return 1;
        }

        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn show_clock_context_menu(hwnd: HWND) {
    debug_log("[gui] right-click context menu shown\n");
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        let hide_text = crate::i18n::tr(crate::i18n::TrKey::ShowClock);
        let hide_wide: Vec<u16> = hide_text.encode_utf16().chain(std::iter::once(0)).collect();
        AppendMenuW(menu, MF_STRING, IDM_HIDE_CLOCK, hide_wide.as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());

        let exit_text = crate::i18n::tr(crate::i18n::TrKey::Exit);
        let exit_wide: Vec<u16> = exit_text.encode_utf16().chain(std::iter::once(0)).collect();
        AppendMenuW(menu, MF_STRING, IDM_EXIT, exit_wide.as_ptr());

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);

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

        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
    }
}

// ───────────────────────────────────────────────
//  Redraw with GDI+ (proper alpha, zero hacks)
// ───────────────────────────────────────────────

// ───────────────────────────────────────────────
//  Animation
// ───────────────────────────────────────────────

unsafe fn process_animation_frame(state: &mut GuiState) {
    let Some(ref anim) = state.animation else {
        return;
    };
    let (duration_ms, fade_dir) = match anim.kind {
        AnimKind::Enter => (SLIDE_IN_MS, 1.0f32),
        AnimKind::Exit => (SLIDE_OUT_MS, -1.0f32),
    };
    let elapsed_ms = anim.start.elapsed().as_millis() as u32;
    let t = (elapsed_ms as f32 / duration_ms as f32).min(1.0);
    let e = ease_out_cubic(t);
    let kind = anim.kind;
    let cur_y = anim.start_y + ((anim.end_y - anim.start_y) as f32 * e) as i32;
    let alpha = if fade_dir > 0.0 {
        (255.0 * e) as u8
    } else {
        (255.0 * (1.0 - e)) as u8
    };

    let hwnd = state.hwnd.as_hwnd();
    let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };

    // Update time and redraw with current alpha.
    let now = Local::now();
    state.last_time_str = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
    unsafe {
        redraw_layered_window_with_alpha(state, alpha);
        SetWindowPos(
            hwnd,
            (-1isize) as HWND,
            (sw - state.width) / 2,
            cur_y,
            state.width,
            state.height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }

    match kind {
        AnimKind::Enter => {
            if t >= 1.0 {
                state.animation = None;
                GUI_VISIBLE.store(true, Ordering::Relaxed);
                state.shown_at = Some(std::time::Instant::now());
                unsafe {
                    KillTimer(hwnd, ANIM_TIMER_ID);
                    SetTimer(hwnd, state.timer_update_id, 500, std::ptr::null_mut());
                }
            }
        }
        AnimKind::Exit => {
            if t >= 1.0 {
                state.animation = None;
                unsafe {
                    KillTimer(hwnd, ANIM_TIMER_ID);
                    ShowWindow(hwnd, SW_HIDE);
                }
                GUI_VISIBLE.store(false, Ordering::Relaxed);
                state.shown_at = None;
                debug_log("[gui] clock hidden\n");
            }
        }
    }
}

unsafe fn redraw_layered_window_with_alpha(state: &mut GuiState, constant_alpha: u8) {
    unsafe {
        let w = state.width;
        let h = state.height;
        let hdc = state.mem_dc.as_hdc();

        {
            let bits = std::slice::from_raw_parts_mut(
                state.bitmap_bits.0 as *mut u8,
                (w * h * 4) as usize,
            );
            bits.fill(0);
        }

        let (r, g, b, opacity_pct) = state.config.bg_color;
        let alpha = ((opacity_pct as f32 / 100.0) * 255.0) as u8;
        let (tr, tg, tb) = state.config.text_color;

        let mut graphics: GpGraphics = std::ptr::null_mut();
        if GdipCreateFromHDC(hdc, &mut graphics) != GDI_PLUS_OK {
            return;
        }

        GdipSetTextRenderingHint(graphics, TEXT_RENDERING_HINT_ANTIALIAS);
        GdipSetSmoothingMode(graphics, SMOOTHING_MODE_HIGH_QUALITY);

        let bg_argb: u32 =
            ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let mut bg_brush: GpBrush = std::ptr::null_mut();
        if GdipCreateSolidFill(bg_argb, &mut bg_brush) == GDI_PLUS_OK {
            GdipFillRectangleI(graphics, bg_brush, 0, 0, w, h);
            GdipDeleteBrush(bg_brush);
        }

        let text_argb: u32 =
            (0xFF_000000u32) | ((tr as u32) << 16) | ((tg as u32) << 8) | (tb as u32);
        let mut text_brush: GpBrush = std::ptr::null_mut();
        if GdipCreateSolidFill(text_argb, &mut text_brush) == GDI_PLUS_OK {
            let text_wide = to_wide(&state.last_time_str);
            let th = state.text_h as f32;
            let y_off = ((h - state.text_h) as f32) / 2.0;
            let layout_rect = RectF {
                x: 0.0,
                y: y_off,
                width: w as f32,
                height: th,
            };
            GdipDrawString(
                graphics,
                text_wide.as_ptr(),
                -1,
                state.gp_font.gp_font(),
                &layout_rect,
                state.gp_string_format.gp_string_format(),
                text_brush,
            );
            GdipDeleteBrush(text_brush);
        }

        GdipDeleteGraphics(graphics);

        let blend = BLENDFUNCTION {
            blend_op: 0,
            blend_flags: 0,
            source_constant_alpha: constant_alpha,
            alpha_format: AC_SRC_ALPHA,
        };
        let pt_src = POINT { x: 0, y: 0 };
        let size = [w, h];
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

unsafe fn redraw_layered_window(state: &mut GuiState) {
    unsafe {
        redraw_layered_window_with_alpha(state, 255);
    }
}

// ───────────────────────
//  Show / hide
// ───────────────────────
//  Show / hide
// ───────────────────────

unsafe fn hide_clock_internal(state: &mut GuiState) {
    unsafe {
        let hwnd = state.hwnd.as_hwnd();

        // Already animating or already hidden.
        if state.animation.is_some() || !GUI_VISIBLE.load(Ordering::Relaxed) {
            return;
        }

        KillTimer(hwnd, state.timer_update_id);

        let mut rect = std::mem::zeroed::<RECT>();
        GetWindowRect(hwnd, &mut rect);
        let cur_y = rect.top;

        state.animation = Some(Animation {
            kind: AnimKind::Exit,
            start: std::time::Instant::now(),
            start_y: cur_y,
            end_y: cur_y - SLIDE_DISTANCE - state.height,
        });
        SetTimer(hwnd, ANIM_TIMER_ID, ANIM_INTERVAL_MS, std::ptr::null_mut());
    }
}

pub fn hide_clock() {
    debug_log("[gui] hide_clock() called\n");
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
    debug_log("[gui] show_clock() called\n");
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

        if state.animation.is_some() {
            return;
        }

        let sh = GetSystemMetrics(SM_CYSCREEN);
        let target_y = sh / 6;

        let now = Local::now();
        state.last_time_str = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());

        state.animation = Some(Animation {
            kind: AnimKind::Enter,
            start: std::time::Instant::now(),
            start_y: target_y - SLIDE_DISTANCE,
            end_y: target_y,
        });
        KillTimer(hwnd, state.timer_update_id);
        SetTimer(hwnd, ANIM_TIMER_ID, ANIM_INTERVAL_MS, std::ptr::null_mut());
    }
}

// ───────────────────────────────────────────────
//  Create
// ───────────────────────────────────────────────

pub fn create_clock_window(cfg: &GeneralConfig) -> Result<(), String> {
    // One-time GDI+ initialisation
    gdiplus_init()?;

    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    if hinst.is_null() {
        return Err("GetModuleHandleW returned NULL".into());
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
        h_cursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
        hbr_background: std::ptr::null_mut(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
        h_icon_sm: std::ptr::null_mut(),
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err("RegisterClassExW failed".into());
    }

    let system_dpi = get_system_dpi();
    let scaled_font_size = FONT_SIZE_PT * (system_dpi / 96.0);

    let screen_dc = unsafe { GetDC(std::ptr::null_mut()) };
    let mut tmp_graphics: GpGraphics = std::ptr::null_mut();
    if unsafe { GdipCreateFromHDC(screen_dc, &mut tmp_graphics) } != GDI_PLUS_OK {
        unsafe { ReleaseDC(std::ptr::null_mut(), screen_dc) };
        return Err("GDI+ Graphics creation failed".into());
    }

    // Create the font for measurement
    let mut gp_family: GpFontFamily = std::ptr::null_mut();
    let font_wide = to_wide(FONT_NAME);
    if unsafe {
        GdipCreateFontFamilyFromName(font_wide.as_ptr(), std::ptr::null_mut(), &mut gp_family)
    } != GDI_PLUS_OK
    {
        return Err("GDI+ font family creation failed".into());
    }

    let mut gp_font: GpFont = std::ptr::null_mut();
    if unsafe {
        GdipCreateFont(
            gp_family,
            scaled_font_size,
            FONT_STYLE_REGULAR,
            UNIT_POINT,
            &mut gp_font,
        )
    } != GDI_PLUS_OK
    {
        unsafe { GdipDeleteFontFamily(gp_family) };
        return Err("GDI+ font creation failed".into());
    }

    let mut gp_sf: GpStringFormat = std::ptr::null_mut();
    if unsafe { GdipCreateStringFormat(0, 0, &mut gp_sf) } != GDI_PLUS_OK {
        unsafe {
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(gp_family);
        }
        return Err("GDI+ string format creation failed".into());
    }
    unsafe {
        GdipSetStringFormatAlign(gp_sf, STRING_ALIGN_CENTER);
        GdipSetStringFormatLineAlign(gp_sf, STRING_ALIGN_CENTER);
    }

    // Measure the display string extent
    let display_wide = to_wide(DISPLAY_STR);
    // Dynamic measurement using a temporary GDI+ bitmap (top-down DIB).
    // Use a generous area to avoid clipping at any DPI.
    let measure_w = 1200i32;
    let measure_h = 200i32;
    let bmi = BITMAPINFOHEADER {
        bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        bi_width: measure_w,
        bi_height: -measure_h, // negative = top-down DIB
        bi_planes: 1,
        bi_bit_count: 32,
        bi_compression: BI_RGB,
        bi_size_image: 0,
        bi_x_pels_per_meter: 0,
        bi_y_pels_per_meter: 0,
        bi_clr_used: 0,
        bi_clr_important: 0,
    };
    let mut tmp_bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let tmp_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    let tmp_bmp = unsafe {
        CreateDIBSection(
            tmp_dc,
            &bmi,
            0,
            &mut tmp_bits as *mut _,
            std::ptr::null_mut(),
            0,
        )
    };
    unsafe { SelectObject(tmp_dc, tmp_bmp as HGDIOBJ) };

    let mut mg: GpGraphics = std::ptr::null_mut();
    let text_w: i32;
    let mut text_h: i32;

    if unsafe { GdipCreateFromHDC(tmp_dc, &mut mg) } == GDI_PLUS_OK {
        // Fill black, draw white text, then scan for white pixels
        let black: u32 = 0xFF_000000u32;
        let white: u32 = 0xFF_FFFFFFu32;
        let mut bb: GpBrush = std::ptr::null_mut();
        let mut wb: GpBrush = std::ptr::null_mut();
        unsafe {
            GdipCreateSolidFill(black, &mut bb);
            GdipCreateSolidFill(white, &mut wb);
            GdipFillRectangleI(mg, bb, 0, 0, measure_w, measure_h);
        }

        let measure_rect = RectF {
            x: 0.0,
            y: 0.0,
            width: measure_w as f32,
            height: measure_h as f32,
        };
        unsafe {
            GdipDrawString(
                mg,
                display_wide.as_ptr(),
                -1,
                gp_font,
                &measure_rect,
                gp_sf,
                wb,
            );
        }

        // Scan for non-black pixels to find bounding box
        let bits = unsafe {
            std::slice::from_raw_parts(tmp_bits as *const u8, (measure_w * measure_h * 4) as usize)
        };
        let mut min_x = measure_w;
        let mut max_x = 0i32;
        let mut min_y = measure_h;
        let mut max_y = 0i32;
        for py in 0..measure_h {
            for px in 0..measure_w {
                let idx = ((py * measure_w + px) * 4) as usize;
                // Check if pixel is non-black (BGRA: any of B,G,R > 0)
                if bits[idx] != 0 || bits[idx + 1] != 0 || bits[idx + 2] != 0 {
                    if px < min_x {
                        min_x = px;
                    }
                    if px > max_x {
                        max_x = px;
                    }
                    if py < min_y {
                        min_y = py;
                    }
                    if py > max_y {
                        max_y = py;
                    }
                }
            }
        }

        if max_x >= min_x && max_y >= min_y {
            text_w = max_x - min_x + 1;
            text_h = max_y - min_y + 1;
        } else {
            // Fallback — reasonable estimate
            text_w = 124;
            text_h = 28;
        }

        // Use GDI+ font height as floor — pixel scan may miss antialiased edges.
        let mut gp_font_h: f32 = 0.0;
        if unsafe { GdipGetFontHeight(gp_font, mg, &mut gp_font_h) } == GDI_PLUS_OK {
            let fh = gp_font_h.ceil() as i32;
            if fh > text_h {
                text_h = fh;
            }
        }

        unsafe {
            GdipDeleteBrush(bb);
            GdipDeleteBrush(wb);
            GdipDeleteGraphics(mg);
        }
    } else {
        text_w = 124;
        text_h = 28;
    }

    unsafe {
        DeleteObject(tmp_bmp as HGDIOBJ);
        DeleteDC(tmp_dc);
    }

    let win_w = text_w + PAD_X * 2;
    let win_h = text_h + PAD_Y * 2;

    // Clean up temporary GDI+ objects
    unsafe {
        GdipDeleteGraphics(tmp_graphics);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
    }

    // ── Create the layered window ──────────────

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
        return Err("CreateWindowExW failed".into());
    }

    // ── DIB section for UpdateLayeredWindow ─────

    let mem_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };

    let bmi = BITMAPINFOHEADER {
        bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        bi_width: win_w,
        bi_height: -win_h, // negative = top-down DIB (consistent with measurement)
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

    let window_config = ClockWindowConfig {
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
        text_h,
        mem_dc: RawPtr::from_ptr(mem_dc),
        bitmap: RawPtr::from_ptr(bitmap),
        bitmap_bits: RawPtr::from_ptr(bitmap_bits),
        gp_font_family: GpObj(gp_family as isize),
        gp_font: GpObj(gp_font as isize),
        gp_string_format: GpObj(gp_sf as isize),
        last_time_str: String::new(),
        timer_update_id: 1,
        animation: None,
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

#[allow(dead_code)]
pub fn destroy_clock_window() {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref state) = *state_opt {
            unsafe {
                KillTimer(state.hwnd.as_hwnd(), state.timer_update_id);
                GdipDeleteFont(state.gp_font.gp_font());
                GdipDeleteFontFamily(state.gp_font_family.gp_font_family());
                GdipDeleteStringFormat(state.gp_string_format.gp_string_format());
                DeleteObject(state.bitmap.as_hgdiobj());
                DeleteDC(state.mem_dc.as_hdc());
                DestroyWindow(state.hwnd.as_hwnd());
            }
        }
        *state_opt = None;
    }
    gdiplus_shutdown();
}

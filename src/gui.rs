#![allow(clippy::upper_case_acronyms)]

use crate::config::GeneralConfig;
use chrono::{Local, Timelike};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
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
    fn GdipMeasureString(
        graphics: GpGraphics,
        string: *const u16,
        length: i32,
        font: GpFont,
        layout_rect: *const RectF,
        string_format: GpStringFormat,
        bounding_box: *mut RectF,
        codepoints_fitted: *mut i32,
        lines_filled: *mut i32,
    ) -> i32;
    fn GdipSetTextRenderingHint(graphics: GpGraphics, mode: i32) -> i32;
    fn GdipSetSmoothingMode(graphics: GpGraphics, mode: i32) -> i32;
}

// ───────────────────────────────────────────────
//  Win32 types & constants (window management)
// ───────────────────────────────────────────────

type HINSTANCE = *mut std::ffi::c_void;
type HWND = *mut std::ffi::c_void;
type HMONITOR = *mut std::ffi::c_void;
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

const MONITOR_DEFAULTTONEAREST: u32 = 2;

const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SW_HIDE: i32 = 0;
const HWND_TOPMOST: isize = -1;

const WM_HOTKEY: u32 = 0x0312;
const WM_USER_HOTKEY: u32 = 0x0401;
const WM_TIMER: u32 = 0x0113;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_SETCURSOR: u32 = 0x0020;
const WM_EXITSIZEMOVE: u32 = 0x0232;
const WM_DPICHANGED: u32 = 0x02E0;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const HTCAPTION: isize = 2;

const IDC_ARROW: *const u16 = 32512usize as *const u16;
const WHITE_BRUSH: i32 = 0;
const OPAQUE_BK: i32 = 2;

const LOGPIXELSY: i32 = 90; // pixels per logical inch (vertical DPI)

// ───────────────────────────────────────────────
//  Win32 structures
// ───────────────────────────────────────────────

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct SIZE {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MONITORINFO {
    cb_size: u32,
    monitor: RECT,
    work: RECT,
    flags: u32,
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
#[allow(non_snake_case)]
struct PAINTSTRUCT {
    hdc: HDC,
    fErase: i32,
    rcPaint: RECT,
    fRestore: i32,
    fIncUpdate: i32,
    rgbReserved: [u8; 32],
}

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HWND,
    h_cursor: HWND,
    hbr_background: HBRUSH,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HWND,
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
    fn UpdateLayeredWindow(
        hWnd: HWND,
        hdcDst: HDC,
        pptDst: *const POINT,
        psize: *const SIZE,
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
    fn MonitorFromPoint(pt: POINT, flags: u32) -> HMONITOR;
    fn GetMonitorInfoW(monitor: HMONITOR, info: *mut MONITORINFO) -> i32;
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
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
    fn AdjustWindowRectEx(lpRect: *mut RECT, dwStyle: u32, bMenu: i32, dwExStyle: u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> i32;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
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
    fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
    fn GetStockObject(fnObject: i32) -> HGDIOBJ;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
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
}

// Handles are stored as integer-sized opaque values. They are never
// dereferenced by Rust and all operations are dispatched on the GUI thread.
#[derive(Debug, Clone, Copy)]
struct RawPtr(isize);

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

#[derive(Debug, Clone, Copy)]
struct GpObj(isize);

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
    // SAFETY: a screen DC is acquired and released in the same scope.
    unsafe {
        let screen_dc = GetDC(std::ptr::null_mut());
        if screen_dc.is_null() {
            return 96.0;
        }
        let dpi = GetDeviceCaps(screen_dc, LOGPIXELSY);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        if dpi > 0 { dpi as f32 } else { 96.0 }
    }
}

fn monitor_work_area(point: POINT) -> Result<RECT, String> {
    // SAFETY: info has the required cb_size and is valid for the call.
    unsafe {
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return Err("MonitorFromPoint failed".into());
        }
        let mut info = MONITORINFO {
            cb_size: std::mem::size_of::<MONITORINFO>() as u32,
            monitor: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            work: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            flags: 0,
        };
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return Err("GetMonitorInfoW failed".into());
        }
        Ok(info.work)
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

// Hardcoded font parameters
const FONT_NAME: &str = "Microsoft YaHei UI";
const FONT_SIZE_PT: f32 = 24.0;
const DISPLAY_STR: &str = "88 : 88 : 88";
const PAD_X: i32 = 0;
const PAD_Y: i32 = 0;
const TEXT_Y_OFFSET: f32 = 2.0; // manual pixel shift to visually centre digit-only text

// ───────────────────────────────────────────────
//  Clock window state
// ───────────────────────────────────────────────

const ANIM_TIMER_ID: usize = 2;
const ANIM_INTERVAL_MS: u32 = 16;
const SLIDE_IN_MS: u32 = 800;
const SLIDE_OUT_MS: u32 = 1500;
const SLIDE_DISTANCE: i32 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimKind {
    Enter,
    Exit,
}

struct Animation {
    kind: AnimKind,
    start: std::time::Instant,
    start_x: i32,
    end_x: i32,
    y: i32,
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
    // GDI objects
    mem_dc: RawPtr,
    bitmap: RawPtr,
    old_bitmap: RawPtr,
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
static GUI_HWND_STATIC: AtomicIsize = AtomicIsize::new(0);
static POSITION_CALLBACK: OnceLock<Box<dyn Fn(i32, i32) + Send + Sync>> = OnceLock::new();

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
                    } else if timer_id == state.timer_update_id {
                        let now = Local::now();
                        let time_str = format!(
                            "{:02} : {:02} : {:02}",
                            now.hour(),
                            now.minute(),
                            now.second()
                        );
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
            if let Some(state_lock) = GUI_STATE.get() {
                let state_opt = state_lock.lock().unwrap();
                if let Some(ref state) = *state_opt
                    && state.animation.is_some()
                {
                    // Ignore drag during animation.
                    return 0;
                }
            }
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

        WM_DPICHANGED => {
            // Windows supplies a work-area-adjusted rectangle for the new DPI.
            let suggested = lparam as *const RECT;
            if !suggested.is_null() {
                // SAFETY: WM_DPICHANGED guarantees lparam points to a RECT for
                // the duration of this synchronous window-procedure call.
                let rect = unsafe { &*suggested };
                unsafe {
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOACTIVATE,
                    );
                }
            }
            return 0;
        }

        WM_EXITSIZEMOVE => {
            // Never invoke application callbacks while holding GUI_STATE: the
            // callback persists config and therefore takes a different lock.
            let position = GUI_STATE.get().and_then(|state_lock| {
                let state_opt = state_lock.lock().unwrap_or_else(|e| e.into_inner());
                state_opt.as_ref().and_then(|state| {
                    let mut rect = RECT {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    };
                    // SAFETY: state owns a live HWND while it is present.
                    (unsafe { GetWindowRect(state.hwnd.as_hwnd(), &mut rect) } != 0)
                        .then_some((rect.left, rect.top))
                })
            });
            if let (Some(cb), Some((x, y))) = (POSITION_CALLBACK.get(), position) {
                cb(x, y);
            }
            return 0;
        }

        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

// ───────────────────────────────────────────────
//  Redraw with GDI+ (proper alpha, zero hacks)
// ───────────────────────────────────────────────

// ───────────────────────────────────────────────
//  Animation
// ───────────────────────────────────────────────

unsafe fn process_animation_frame(state: &mut GuiState) {
    // Extract animation state first, releasing the immutable borrow before mutable ops.
    let (kind, cur_x, anim_y, t, alpha, anchor_x) = {
        let Some(ref anim) = state.animation else {
            return;
        };
        let duration_ms = match anim.kind {
            AnimKind::Enter => SLIDE_IN_MS,
            AnimKind::Exit => SLIDE_OUT_MS,
        };
        let elapsed_ms = anim.start.elapsed().as_millis() as u32;
        let t = (elapsed_ms as f32 / duration_ms as f32).min(1.0);
        let e = ease_out_cubic(t);
        let kind = anim.kind;
        let cur_x = anim.start_x + ((anim.end_x - anim.start_x) as f32 * e) as i32;
        let anim_y = anim.y;
        let anchor_x = match kind {
            AnimKind::Enter => anim.end_x, // slide-right target = visible position
            AnimKind::Exit => anim.start_x, // slide-left origin = visible position
        };
        let alpha: u8 = match kind {
            AnimKind::Enter => (255.0 * e) as u8,
            AnimKind::Exit => (255.0 * (1.0 - e)) as u8,
        };
        (kind, cur_x, anim_y, t, alpha, anchor_x)
    };

    let hwnd = state.hwnd.as_hwnd();

    // Update time and redraw with current alpha.
    let now = Local::now();
    let new_time = format!(
        "{:02} : {:02} : {:02}",
        now.hour(),
        now.minute(),
        now.second()
    );
    if new_time != state.last_time_str {
        state.last_time_str = new_time;
    }
    unsafe {
        redraw_layered_window_with_alpha(state, alpha);
        SetWindowPos(
            hwnd,
            HWND_TOPMOST as HWND,
            cur_x,
            anim_y,
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
                    // Hide first, then restore position so next show starts from anchor.
                    ShowWindow(hwnd, SW_HIDE);
                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST as HWND,
                        anchor_x,
                        anim_y,
                        state.width,
                        state.height,
                        SWP_NOACTIVATE,
                    );
                    KillTimer(hwnd, ANIM_TIMER_ID);
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
            // Shift the layout rectangle down by a fixed offset so digit-only
            // strings (no ascenders/descenders) appear visually centred.
            let layout_rect = RectF {
                x: 0.0,
                y: TEXT_Y_OFFSET,
                width: w as f32,
                height: h as f32,
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
        let size = SIZE { cx: w, cy: h };
        let screen_dc = GetDC(std::ptr::null_mut());
        UpdateLayeredWindow(
            state.hwnd.as_hwnd(),
            screen_dc,
            std::ptr::null(),
            &size,
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

unsafe fn hide_clock_internal(state: &mut GuiState) {
    unsafe {
        let hwnd = state.hwnd.as_hwnd();

        // Already animating or already hidden.
        if state.animation.is_some() || !GUI_VISIBLE.load(Ordering::Relaxed) {
            return;
        }

        KillTimer(hwnd, state.timer_update_id);

        // Read current window position (real-time, not from initial placement)
        let mut rect = std::mem::zeroed::<RECT>();
        GetWindowRect(hwnd, &mut rect);
        let cur_x = rect.left;
        let cur_y = rect.top;

        state.animation = Some(Animation {
            kind: AnimKind::Exit,
            start: std::time::Instant::now(),
            start_x: cur_x,
            end_x: cur_x - SLIDE_DISTANCE,
            y: cur_y,
        });
        if SetTimer(hwnd, ANIM_TIMER_ID, ANIM_INTERVAL_MS, std::ptr::null_mut()) == 0 {
            debug_log("[gui] SetTimer(ANIM) failed, animation won't run\n");
            state.animation = None;
            ShowWindow(hwnd, SW_HIDE);
            GUI_VISIBLE.store(false, Ordering::Relaxed);
            state.shown_at = None;
        }
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

        // If an exit animation is in progress (e.g. schedule fires while the
        // user is manually hiding the clock), cancel it so we can restart a
        // fresh enter animation immediately.
        if let Some(ref anim) = state.animation {
            if anim.kind == AnimKind::Exit {
                KillTimer(hwnd, ANIM_TIMER_ID);
                state.animation = None;
            } else {
                // Enter animation already running — nothing to do.
                return;
            }
        }

        // Fully visible with no animation — just refresh the auto-hide timer.
        if GUI_VISIBLE.load(Ordering::Relaxed) {
            state.shown_at = Some(std::time::Instant::now());
            return;
        }

        // Read current window position (valid even when hidden).
        // This ensures the slide-in starts from the last-dragged position.
        let mut rect = std::mem::zeroed::<RECT>();
        GetWindowRect(hwnd, &mut rect);
        let cur_x = rect.left;
        let cur_y = rect.top;

        let now = Local::now();
        state.last_time_str = format!(
            "{:02}：{:02}：{:02}",
            now.hour(),
            now.minute(),
            now.second()
        );

        state.animation = Some(Animation {
            kind: AnimKind::Enter,
            start: std::time::Instant::now(),
            start_x: cur_x - SLIDE_DISTANCE,
            end_x: cur_x,
            y: cur_y,
        });
        KillTimer(hwnd, state.timer_update_id);
        if SetTimer(hwnd, ANIM_TIMER_ID, ANIM_INTERVAL_MS, std::ptr::null_mut()) == 0 {
            debug_log("[gui] SetTimer(ANIM) failed, showing clock immediately\n");
            state.animation = None;
            GUI_VISIBLE.store(true, Ordering::Relaxed);
            state.shown_at = Some(std::time::Instant::now());
            SetTimer(hwnd, state.timer_update_id, 500, std::ptr::null_mut());
        }
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

    // UNIT_POINT already converts points using the target HDC's DPI. Scaling
    // the point size again would make text grow quadratically at high DPI.
    let scaled_font_size = FONT_SIZE_PT;

    let screen_dc = unsafe { GetDC(std::ptr::null_mut()) };
    let mut tmp_graphics: GpGraphics = std::ptr::null_mut();
    if unsafe { GdipCreateFromHDC(screen_dc, &mut tmp_graphics) } != GDI_PLUS_OK {
        unsafe { ReleaseDC(std::ptr::null_mut(), screen_dc) };
        return Err("GDI+ Graphics creation failed".into());
    }

    // Helper to clean up screen DC on error paths
    let cleanup_screen_dc = || unsafe { ReleaseDC(std::ptr::null_mut(), screen_dc) };

    // Try preferred font first, fall back to system default on failure
    let mut gp_family: GpFontFamily = std::ptr::null_mut();
    let font_wide = to_wide(FONT_NAME);
    if unsafe {
        GdipCreateFontFamilyFromName(font_wide.as_ptr(), std::ptr::null_mut(), &mut gp_family)
    } != GDI_PLUS_OK
    {
        // Fallback: use generic sans-serif (pass empty string = system default)
        let fallback_wide = to_wide("");
        if unsafe {
            GdipCreateFontFamilyFromName(
                fallback_wide.as_ptr(),
                std::ptr::null_mut(),
                &mut gp_family,
            )
        } != GDI_PLUS_OK
        {
            cleanup_screen_dc();
            return Err("GDI+ font family creation failed".into());
        }
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
        unsafe {
            GdipDeleteFontFamily(gp_family);
            GdipDeleteGraphics(tmp_graphics);
        }
        cleanup_screen_dc();
        return Err("GDI+ font creation failed".into());
    }

    let mut gp_sf: GpStringFormat = std::ptr::null_mut();
    if unsafe { GdipCreateStringFormat(0, 0, &mut gp_sf) } != GDI_PLUS_OK {
        unsafe {
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(gp_family);
            GdipDeleteGraphics(tmp_graphics);
        }
        cleanup_screen_dc();
        return Err("GDI+ string format creation failed".into());
    }
    unsafe {
        GdipSetStringFormatAlign(gp_sf, STRING_ALIGN_CENTER);
        GdipSetStringFormatLineAlign(gp_sf, STRING_ALIGN_CENTER);
    }

    // Vertical offset for digit-only text: GDI+ centres the font cell,
    // digits have no ascenders/descenders so they sit slightly high.
    // A fixed +2 px shift is applied at render time (see TEXT_Y_OFFSET).
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
    if tmp_dc.is_null() || tmp_bmp.is_null() || tmp_bits.is_null() {
        unsafe {
            if !tmp_bmp.is_null() {
                DeleteObject(tmp_bmp as HGDIOBJ);
            }
            if !tmp_dc.is_null() {
                DeleteDC(tmp_dc);
            }
            GdipDeleteStringFormat(gp_sf);
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(gp_family);
            GdipDeleteGraphics(tmp_graphics);
        }
        cleanup_screen_dc();
        return Err("Failed to create temporary measurement bitmap".into());
    }
    let tmp_old_bmp = unsafe { SelectObject(tmp_dc, tmp_bmp as HGDIOBJ) };
    if tmp_old_bmp.is_null() {
        unsafe {
            DeleteObject(tmp_bmp as HGDIOBJ);
            DeleteDC(tmp_dc);
            GdipDeleteStringFormat(gp_sf);
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(gp_family);
            GdipDeleteGraphics(tmp_graphics);
        }
        cleanup_screen_dc();
        return Err("SelectObject failed for measurement bitmap".into());
    }

    let mut mg: GpGraphics = std::ptr::null_mut();
    let text_w: i32;
    let text_h: i32;

    if unsafe { GdipCreateFromHDC(tmp_dc, &mut mg) } == GDI_PLUS_OK {
        // Use GDI+ measurement APIs instead of pixel scanning. This
        // provides reliable font metrics (including ascent/descent/bearing)
        // and avoids raster-based inaccuracies across DPIs.
        let layout_rect = RectF {
            x: 0.0,
            y: 0.0,
            width: measure_w as f32,
            height: measure_h as f32,
        };
        let mut bounding = RectF {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let mut codepoints_fitted: i32 = 0;
        let mut lines_filled: i32 = 0;
        let status = unsafe {
            GdipMeasureString(
                mg,
                display_wide.as_ptr(),
                -1,
                gp_font,
                &layout_rect,
                gp_sf,
                &mut bounding as *mut RectF,
                &mut codepoints_fitted as *mut i32,
                &mut lines_filled as *mut i32,
            )
        };

        if status == GDI_PLUS_OK && bounding.width > 0.0 && bounding.height > 0.0 {
            // Convert measured floating size to integer pixels and add
            // a small padding to account for antialiasing and glyph overhangs.
            const WIDTH_PAD: i32 = 4;
            const HEIGHT_PAD: i32 = 2;
            text_w = bounding.width.ceil() as i32 + WIDTH_PAD;
            text_h = bounding.height.ceil() as i32 + HEIGHT_PAD;
        } else {
            // Fallback — conservative window size
            text_w = 240;
            text_h = 60;
        }

        unsafe {
            GdipDeleteGraphics(mg);
        }
    } else {
        // Fallback — conservative window size
        text_w = 240;
        text_h = 60;
    }

    unsafe {
        SelectObject(tmp_dc, tmp_old_bmp);
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

    // ── Determine initial window position ──────

    let desired = if cfg.window_x == -1 || cfg.window_y == -1 {
        POINT { x: 0, y: 0 }
    } else {
        POINT {
            x: cfg.window_x,
            y: cfg.window_y,
        }
    };
    let work = monitor_work_area(desired)?;
    let (init_x, init_y) = if cfg.window_x == -1 || cfg.window_y == -1 {
        // Centre in the primary monitor work area, excluding the taskbar.
        (
            work.left + (work.right - work.left - win_w) / 2,
            work.top + (work.bottom - work.top) / 6,
        )
    } else {
        // Negative coordinates are valid on monitors left/above the primary.
        // Clamp the complete window into the nearest monitor's work area.
        (
            cfg.window_x
                .clamp(work.left, (work.right - win_w).max(work.left)),
            cfg.window_y
                .clamp(work.top, (work.bottom - win_h).max(work.top)),
        )
    };

    // ── Create the layered window ──────────────

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            std::ptr::null(),
            WS_POPUP,
            init_x,
            init_y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        // Clean up previously allocated resources
        unsafe {
            if !gp_sf.is_null() {
                GdipDeleteStringFormat(gp_sf);
            }
            if !gp_font.is_null() {
                GdipDeleteFont(gp_font);
            }
            if !gp_family.is_null() {
                GdipDeleteFontFamily(gp_family);
            }
        }
        return Err("CreateWindowExW failed".into());
    }

    // ── DIB section for UpdateLayeredWindow ─────

    let mem_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if mem_dc.is_null() {
        // Clean up previously allocated resources
        unsafe {
            DestroyWindow(hwnd);
            if !gp_sf.is_null() {
                GdipDeleteStringFormat(gp_sf);
            }
            if !gp_font.is_null() {
                GdipDeleteFont(gp_font);
            }
            if !gp_family.is_null() {
                GdipDeleteFontFamily(gp_family);
            }
        }
        return Err("CreateCompatibleDC failed".into());
    }

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
    if bitmap.is_null() || bitmap_bits.is_null() {
        // Clean up previously allocated resources
        unsafe {
            DeleteDC(mem_dc);
            DestroyWindow(hwnd);
            if !gp_sf.is_null() {
                GdipDeleteStringFormat(gp_sf);
            }
            if !gp_font.is_null() {
                GdipDeleteFont(gp_font);
            }
            if !gp_family.is_null() {
                GdipDeleteFontFamily(gp_family);
            }
        }
        return Err("CreateDIBSection failed".into());
    }
    let old_bitmap = unsafe { SelectObject(mem_dc, bitmap as HGDIOBJ) };
    if old_bitmap.is_null() {
        unsafe {
            DeleteObject(bitmap as HGDIOBJ);
            DeleteDC(mem_dc);
            DestroyWindow(hwnd);
            GdipDeleteStringFormat(gp_sf);
            GdipDeleteFont(gp_font);
            GdipDeleteFontFamily(gp_family);
        }
        return Err("SelectObject failed for clock bitmap".into());
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
        mem_dc: RawPtr::from_ptr(mem_dc),
        bitmap: RawPtr::from_ptr(bitmap),
        old_bitmap: RawPtr::from_ptr(old_bitmap),
        bitmap_bits: RawPtr::from_ptr(bitmap_bits),
        gp_font_family: GpObj(gp_family as isize),
        gp_font: GpObj(gp_font as isize),
        gp_string_format: GpObj(gp_sf as isize),
        last_time_str: String::new(),
        timer_update_id: 1,
        animation: None,
    };

    GUI_STATE.set(Mutex::new(Some(state))).ok();
    GUI_HWND_STATIC.store(hwnd as isize, Ordering::Relaxed);

    Ok(())
}

pub fn get_hwnd() -> HWND {
    GUI_HWND_STATIC.load(Ordering::Relaxed) as *mut std::ffi::c_void
}

pub fn is_visible() -> bool {
    GUI_VISIBLE.load(Ordering::Relaxed)
}

/// Register a callback to be invoked when the user finishes dragging the window.
/// The callback receives the new (x, y) position of the top-left corner.
pub fn set_position_callback<F: Fn(i32, i32) + Send + Sync + 'static>(f: F) {
    POSITION_CALLBACK.set(Box::new(f)).ok();
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

/// Update only the background opacity and redraw if visible.
pub fn update_opacity(opacity: u8) {
    if let Some(state_lock) = GUI_STATE.get() {
        let mut state_opt = state_lock.lock().unwrap();
        if let Some(ref mut state) = *state_opt {
            state.config.bg_color.3 = opacity;
            if GUI_VISIBLE.load(Ordering::Relaxed) {
                unsafe {
                    redraw_layered_window(state);
                }
            }
        }
    }
}

/// Get the current background opacity from GUI state.
pub fn get_current_opacity() -> u8 {
    if let Some(state_lock) = GUI_STATE.get() {
        let state_opt = state_lock.lock().unwrap();
        if let Some(ref state) = *state_opt {
            return state.config.bg_color.3;
        }
    }
    0
}

fn destroy_clock_window() {
    // Remove the state before DestroyWindow, because DestroyWindow synchronously
    // sends WM_DESTROY and the window procedure may consult GUI_STATE.
    let state = GUI_STATE
        .get()
        .and_then(|state_lock| state_lock.lock().unwrap_or_else(|e| e.into_inner()).take());
    if let Some(state) = state {
        // SAFETY: all handles are owned by state and cleanup occurs on the GUI
        // thread. The original selected bitmap is restored before deletion.
        unsafe {
            KillTimer(state.hwnd.as_hwnd(), state.timer_update_id);
            KillTimer(state.hwnd.as_hwnd(), ANIM_TIMER_ID);
            GdipDeleteFont(state.gp_font.gp_font());
            GdipDeleteFontFamily(state.gp_font_family.gp_font_family());
            GdipDeleteStringFormat(state.gp_string_format.gp_string_format());
            SelectObject(state.mem_dc.as_hdc(), state.old_bitmap.as_hgdiobj());
            DeleteObject(state.bitmap.as_hgdiobj());
            DeleteDC(state.mem_dc.as_hdc());
            DestroyWindow(state.hwnd.as_hwnd());
        }
    }
    GUI_HWND_STATIC.store(0, Ordering::Release);
    GUI_VISIBLE.store(false, Ordering::Release);
    gdiplus_shutdown();
}

pub fn destroy_windows() {
    let panel = OPACITY_PANEL_HWND.swap(0, Ordering::AcqRel) as HWND;
    if !panel.is_null() {
        // SAFETY: panel is the live modeless window created by this module.
        unsafe { DestroyWindow(panel) };
    }
    let font = OPACITY_PANEL_FONT.swap(0, Ordering::AcqRel) as HGDIOBJ;
    if !font.is_null() {
        // SAFETY: font was created by CreateFontW and is no longer selected
        // after its child window has been destroyed.
        unsafe { DeleteObject(font) };
    }
    destroy_clock_window();
}

// ───────────────────────────────────────────────
//  Opacity panel — modeless floating window
// ───────────────────────────────────────────────

const WS_CAPTION_WIN: u32 = 0x00C0_0000;
const WS_SYSMENU_WIN: u32 = 0x0008_0000;

const WM_ACTIVATE: u32 = 0x0006;
const WM_COMMAND: u32 = 0x0111;
const EN_CHANGE: u32 = 0x0300;
const ES_NUMBER: u32 = 0x2000;
const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_BORDER: u32 = 0x0080_0000;
const WS_TABSTOP: u32 = 0x0001_0000;
const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
const SW_SHOW: i32 = 5;
const WM_SETFONT: u32 = 0x0030;
const PANEL_FONT_SIZE_PT: f32 = 11.0;

type HFONT = *mut std::ffi::c_void;

// Child control ID for the opacity panel
const IDC_OPACITY_EDIT: isize = 101;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;
    fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn IsWindowVisible(hWnd: HWND) -> i32;
    fn SetFocus(hWnd: HWND) -> HWND;
}

static OPACITY_PANEL_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static OPACITY_PANEL_FONT: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static OPACITY_CLOSE_CALLBACK: OnceLock<Box<dyn Fn(u8) + Send + Sync>> = OnceLock::new();

/// Register a callback invoked when the opacity panel is dismissed.
/// The callback receives the final opacity value (0–100).
pub fn set_opacity_close_callback<F: Fn(u8) + Send + Sync + 'static>(f: F) {
    OPACITY_CLOSE_CALLBACK.set(Box::new(f)).ok();
}

fn fire_opacity_close() {
    if let Some(cb) = OPACITY_CLOSE_CALLBACK.get() {
        cb(get_current_opacity());
    }
}

// ───────────────────────────────────────────────
//  Panel layout constants (96 DPI reference)
// ───────────────────────────────────────────────

const PANEL_CLIENT_W: i32 = 420;
const PANEL_CLIENT_H: i32 = 120;
const PANEL_PAD: i32 = 20;

// Colours
const PANEL_BG: u32 = 0xFF_F5F5F5u32;
const PANEL_TEXT: u32 = 0xFF_333333u32;

// ───────────────────────────────────────────────
//  GDI+ panel drawing
// ───────────────────────────────────────────────

fn panel_paint(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT {
            hdc: std::ptr::null_mut(),
            fErase: 0,
            rcPaint: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            fRestore: 0,
            fIncUpdate: 0,
            rgbReserved: [0u8; 32],
        };
        let hdc = BeginPaint(hwnd, &mut ps);
        if hdc.is_null() {
            return;
        }

        let mut graphics: GpGraphics = std::ptr::null_mut();
        if GdipCreateFromHDC(hdc, &mut graphics) != GDI_PLUS_OK {
            EndPaint(hwnd, &ps);
            return;
        }
        GdipSetTextRenderingHint(graphics, TEXT_RENDERING_HINT_ANTIALIAS);
        GdipSetSmoothingMode(graphics, SMOOTHING_MODE_HIGH_QUALITY);

        // ── Background fill ──
        let mut bg_brush: GpBrush = std::ptr::null_mut();
        GdipCreateSolidFill(PANEL_BG, &mut bg_brush);
        let mut rc_client = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetClientRect(hwnd, &mut rc_client);
        let cw = rc_client.right - rc_client.left;
        let ch = rc_client.bottom - rc_client.top;
        GdipFillRectangleI(graphics, bg_brush, 0, 0, cw, ch);
        GdipDeleteBrush(bg_brush);

        // ── Font & brushes ──
        let dpi = get_system_dpi();
        let scale = dpi / 96.0;
        let scaled_pt = PANEL_FONT_SIZE_PT * scale;
        let face_w = to_wide(FONT_NAME);
        let mut family: GpFontFamily = std::ptr::null_mut();
        let mut gp_font: GpFont = std::ptr::null_mut();
        if GdipCreateFontFamilyFromName(face_w.as_ptr(), std::ptr::null_mut(), &mut family)
            == GDI_PLUS_OK
        {
            GdipCreateFont(
                family,
                scaled_pt,
                FONT_STYLE_REGULAR,
                UNIT_POINT,
                &mut gp_font,
            );
        }

        let mut fmt_left: GpStringFormat = std::ptr::null_mut();
        GdipCreateStringFormat(0, 0, &mut fmt_left);
        if !fmt_left.is_null() {
            GdipSetStringFormatAlign(fmt_left, 0);
            GdipSetStringFormatLineAlign(fmt_left, 0);
        }

        let mut text_brush: GpBrush = std::ptr::null_mut();
        GdipCreateSolidFill(PANEL_TEXT, &mut text_brush);

        let pad = (PANEL_PAD as f32 * scale) as i32;

        // ── Description (auto-wrapped, 2 lines) ──
        let desc_text = match crate::i18n::lang() {
            crate::i18n::Lang::Zh => {
                "调整时钟窗口的背景不透明度。\n0 = 完全透明（不可见）  ·  100 = 完全不透明（纯色背景）"
            }
            _ => {
                "Adjust the background opacity of the clock overlay.\n0 = fully transparent (invisible)  ·  100 = fully opaque (solid)"
            }
        };
        let desc_w = to_wide(desc_text);
        let desc_y = pad;
        let desc_h = (48.0 * scale) as i32;
        let desc_rect = RectF {
            x: pad as f32,
            y: desc_y as f32,
            width: (cw - 2 * pad) as f32,
            height: desc_h as f32,
        };
        if !gp_font.is_null() && !fmt_left.is_null() && !text_brush.is_null() {
            GdipDrawString(
                graphics,
                desc_w.as_ptr(),
                -1,
                gp_font,
                &desc_rect,
                fmt_left,
                text_brush,
            );
        }

        // ── Label ──
        let label_y = desc_y + desc_h + (10.0 * scale) as i32;
        let label_text = match crate::i18n::lang() {
            crate::i18n::Lang::Zh => "当前不透明度：",
            _ => "Current opacity:",
        };
        let label_w = to_wide(label_text);
        let label_rect = RectF {
            x: pad as f32,
            y: label_y as f32,
            width: 145.0 * scale,
            height: 24.0 * scale,
        };
        if !gp_font.is_null() && !fmt_left.is_null() && !text_brush.is_null() {
            GdipSetStringFormatLineAlign(fmt_left, STRING_ALIGN_CENTER);
            GdipDrawString(
                graphics,
                label_w.as_ptr(),
                -1,
                gp_font,
                &label_rect,
                fmt_left,
                text_brush,
            );
            GdipSetStringFormatLineAlign(fmt_left, 0);
        }

        // ── Cleanup ──
        if !gp_font.is_null() {
            GdipDeleteFont(gp_font);
        }
        if !family.is_null() {
            GdipDeleteFontFamily(family);
        }
        if !fmt_left.is_null() {
            GdipDeleteStringFormat(fmt_left);
        }
        if !text_brush.is_null() {
            GdipDeleteBrush(text_brush);
        }

        GdipDeleteGraphics(graphics);
        EndPaint(hwnd, &ps);
    }
}

/// Create the modeless opacity panel (hidden).
/// Must be called once from the main thread after `create_clock_window`.
pub fn create_opacity_panel() -> Result<(), String> {
    let class_w = to_wide("TipClockOpacityPanel");

    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    if hinst.is_null() {
        return Err("GetModuleHandleW returned NULL".into());
    }

    let wc = WNDCLASSEXW {
        cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfn_wnd_proc: opacity_panel_wndproc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinst,
        h_icon: std::ptr::null_mut(),
        h_cursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
        hbr_background: std::ptr::null_mut(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_w.as_ptr(),
        h_icon_sm: std::ptr::null_mut(),
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err("RegisterClassExW failed for opacity panel".into());
    }

    // ── DPI-aware sizing ──
    let dpi = get_system_dpi();
    let scale = dpi / 96.0;

    let client_w: i32 = (PANEL_CLIENT_W as f32 * scale) as i32;
    let client_h: i32 = (PANEL_CLIENT_H as f32 * scale) as i32;
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: client_w,
        bottom: client_h,
    };
    unsafe {
        AdjustWindowRectEx(&mut rc, WS_CAPTION_WIN | WS_SYSMENU_WIN, 0, 0);
    }
    let dlg_w = rc.right - rc.left;
    let dlg_h = rc.bottom - rc.top;

    let work = monitor_work_area(POINT { x: 0, y: 0 })?;
    let dlg_x = work.left + (work.right - work.left - dlg_w) / 2;
    let dlg_y = work.top + (work.bottom - work.top - dlg_h) / 3;

    let title_text = match crate::i18n::lang() {
        crate::i18n::Lang::Zh => "不透明度",
        _ => "Opacity",
    };
    let title_w = to_wide(title_text);

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_w.as_ptr(),
            title_w.as_ptr(),
            WS_CAPTION_WIN | WS_SYSMENU_WIN,
            dlg_x,
            dlg_y,
            dlg_w,
            dlg_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        return Err("CreateWindowExW failed for opacity panel".into());
    }

    // ── DPI-scaled font for the edit control ──
    // CreateFontW expects a logical pixel height, unlike GDI+ UNIT_POINT.
    let scaled_font_size = PANEL_FONT_SIZE_PT * scale;
    let font_height = -((scaled_font_size * 96.0 / 72.0).round() as i32);
    let font_face = to_wide(FONT_NAME);
    let ui_font = unsafe {
        CreateFontW(
            font_height,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            font_face.as_ptr(),
        )
    };

    unsafe {
        let edit_x = (PANEL_PAD as f32 * scale) as i32 + (150.0 * scale) as i32;
        let edit_y =
            (PANEL_PAD as f32 * scale) as i32 + (48.0 * scale) as i32 + (10.0 * scale) as i32;
        let edit_w = (70.0 * scale) as i32;
        let edit_h = (26.0 * scale) as i32;

        let edit = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            to_wide("EDIT").as_ptr(),
            to_wide(&get_current_opacity().to_string()).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | ES_NUMBER,
            edit_x,
            edit_y,
            edit_w,
            edit_h,
            hwnd,
            IDC_OPACITY_EDIT as *mut std::ffi::c_void as HINSTANCE,
            hinst,
            std::ptr::null_mut(),
        );
        if !ui_font.is_null() {
            SendMessageW(edit, WM_SETFONT, ui_font as WPARAM, 1);
        }
    }

    OPACITY_PANEL_HWND.store(hwnd as isize, Ordering::Release);
    OPACITY_PANEL_FONT.store(ui_font as isize, Ordering::Release);
    Ok(())
}

/// Toggle opacity panel visibility.
pub fn toggle_opacity_panel() {
    let hwnd = OPACITY_PANEL_HWND.load(Ordering::Relaxed) as HWND;
    if hwnd.is_null() {
        return;
    }
    unsafe {
        if IsWindowVisible(hwnd) != 0 {
            ShowWindow(hwnd, SW_HIDE);
            fire_opacity_close();
        } else {
            let edit = GetDlgItem(hwnd, IDC_OPACITY_EDIT as i32);
            if !edit.is_null() {
                let text = format!("{}", get_current_opacity());
                SetWindowTextW(edit, to_wide(&text).as_ptr());
            }
            ShowWindow(hwnd, SW_SHOW);
            let edit = GetDlgItem(hwnd, IDC_OPACITY_EDIT as i32);
            if !edit.is_null() {
                SetFocus(edit);
                SendMessageW(edit, 0x00B1, 0, -1); // EM_SETSEL — select all
            }
        }
    }
}

/// Read the edit control, parse value (clamp 0–100), update opacity, and repaint.
fn update_opacity_from_edit(hwnd: HWND) {
    let edit = unsafe { GetDlgItem(hwnd, IDC_OPACITY_EDIT as i32) };
    if edit.is_null() {
        return;
    }
    let mut buf = [0u16; 8];
    let len = unsafe { GetWindowTextW(edit, buf.as_mut_ptr(), buf.len() as i32) };
    if len == 0 {
        return;
    }
    let text = String::from_utf16_lossy(&buf[..len as usize]);
    if let Ok(val) = text.parse::<u32>() {
        update_opacity(val.min(100) as u8);
    }
}

unsafe extern "system" fn opacity_panel_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let code = ((wparam >> 16) & 0xFFFF) as u32;
            let ctrl_id = (wparam & 0xFFFF) as isize;
            if ctrl_id == IDC_OPACITY_EDIT && code == EN_CHANGE {
                update_opacity_from_edit(hwnd);
            }
            return 0;
        }
        WM_PAINT => {
            panel_paint(hwnd);
            return 0;
        }
        WM_ERASEBKGND => {
            return 1;
        }
        WM_CTLCOLOREDIT => {
            // EDIT controls must erase their client area when text changes.
            // A transparent background with NULL_BRUSH leaves old glyph pixels
            // behind, producing visible input trails.
            let hdc = wparam as HDC;
            unsafe {
                SetBkMode(hdc, OPAQUE_BK);
            }
            return unsafe { GetStockObject(WHITE_BRUSH) } as LRESULT;
        }
        WM_ACTIVATE => {
            let code = (wparam as u32) & 0xFFFF;
            if code == 0 {
                unsafe { ShowWindow(hwnd, SW_HIDE) };
                fire_opacity_close();
            }
            return 0;
        }
        WM_CLOSE => {
            unsafe { ShowWindow(hwnd, SW_HIDE) };
            fire_opacity_close();
            return 0;
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

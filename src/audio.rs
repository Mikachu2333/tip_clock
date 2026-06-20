use crate::config::RingType;

#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(pszSound: *const u16, hmod: *mut std::ffi::c_void, fdwSound: u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringW(text: *const u16);
    fn WriteConsoleW(
        console: *mut std::ffi::c_void,
        text: *const u16,
        len: u32,
        written: *mut u32,
        reserved: *mut std::ffi::c_void,
    ) -> i32;
    fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
}

const SND_MEMORY: u32 = 0x0004;
const SND_ASYNC: u32 = 0x0001;
const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5u32; // -11

const START_WAV: &[u8] = include_bytes!("../res/start.wav");
const END_WAV: &[u8] = include_bytes!("../res/end.wav");
const SPECIAL_WAV: &[u8] = include_bytes!("../res/special.wav");

pub struct AudioPlayer;

impl AudioPlayer {
    pub fn new() -> Self {
        AudioPlayer
    }

    pub fn play(&self, ring: RingType) {
        let data = match ring {
            RingType::Start => START_WAV,
            RingType::End => END_WAV,
            RingType::Special => SPECIAL_WAV,
        };
        if !data.is_empty() {
            unsafe {
                PlaySoundW(
                    data.as_ptr() as *const u16,
                    std::ptr::null_mut(),
                    SND_MEMORY | SND_ASYNC,
                );
            }
        }
    }
}

pub(crate) fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn debug_log(s: &str) {
    let wide = to_wide(s);
    unsafe {
        OutputDebugStringW(wide.as_ptr());
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() {
            let mut written: u32 = 0;
            WriteConsoleW(handle, wide.as_ptr(), (wide.len() - 1) as u32, &mut written, std::ptr::null_mut());
        }
    }
}

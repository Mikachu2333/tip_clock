use crate::config::RingType;
// ───────────────────────────────────────────────
//  WinMM FFI
// ───────────────────────────────────────────────

#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(pszSound: *const u16, hmod: *mut std::ffi::c_void, fdwSound: u32) -> i32;
    fn waveOutSetVolume(hwo: *mut std::ffi::c_void, dwVolume: u32) -> u32;
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
const SND_FILENAME: u32 = 0x00020000;
const SND_NODEFAULT: u32 = 0x0002;
const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5u32;

// Embedded default WAVs
const START_WAV: &[u8] = include_bytes!("../res/start.wav");
const END_WAV: &[u8] = include_bytes!("../res/end.wav");
const SPECIAL_WAV: &[u8] = include_bytes!("../res/special.wav");

// ───────────────────────────────────────────────
//  Audio player
// ───────────────────────────────────────────────

pub struct AudioPlayer;

impl AudioPlayer {
    pub fn new(default_volume: u8) -> Self {
        Self::set_wave_volume(default_volume.min(100));
        AudioPlayer
    }

    fn set_wave_volume(vol: u8) {
        // Convert 0-100 to 0-65535 (left + right channels)
        let v = ((vol as f32 / 100.0) * 65535.0) as u32;
        let dw = v | (v << 16);
        unsafe {
            let result = waveOutSetVolume(std::ptr::null_mut(), dw);
            if result != 0 {
                debug_log(format!("[tip_clock] waveOutSetVolume failed: {result}\n"));
            }
        }
    }

    /// Play a ring sound. Returns immediately (async).
    pub fn play(&self, ring: RingType, custom_file: Option<&str>, exe_dir: &std::path::Path) {
        match ring {
            RingType::None => {
                // Do nothing
            }
            RingType::Custom => {
                if let Some(filename) = custom_file {
                    self.play_custom(filename, exe_dir);
                }
            }
            _ => {
                let data = match ring {
                    RingType::Start => START_WAV,
                    RingType::End => END_WAV,
                    RingType::Special => SPECIAL_WAV,
                    _ => unreachable!(),
                };
                if !data.is_empty() {
                    unsafe {
                        let ok = PlaySoundW(
                            data.as_ptr() as *const u16,
                            std::ptr::null_mut(),
                            SND_MEMORY | SND_ASYNC,
                        );
                        if ok == 0 {
                            debug_log("[tip_clock] PlaySoundW (embedded) failed\n");
                        }
                    }
                }
            }
        }
    }

    fn play_custom(&self, filename: &str, exe_dir: &std::path::Path) {
        // Auto-correct: add .wav extension if missing
        let corrected = if filename.to_lowercase().ends_with(".wav") {
            filename.to_string()
        } else {
            format!("{filename}.wav")
        };

        let full_path = exe_dir.join(&corrected);
        if !full_path.exists() {
            debug_log(format!(
                "[tip_clock] custom audio file not found: {}\n",
                full_path.display()
            ));
            return;
        }

        let wide = to_wide(&full_path.to_string_lossy());
        unsafe {
            let ok = PlaySoundW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
            );
            if ok == 0 {
                debug_log(format!(
                    "[tip_clock] PlaySoundW (custom) failed: {}\n",
                    full_path.display()
                ));
            }
        }
    }
}

// ───────────────────────────────────────────────
//  UTF-16 conversion + debug output helpers
// ───────────────────────────────────────────────

pub(crate) fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn debug_log(s: impl ToString) {
    if cfg!(not(debug_assertions)) {
        return;
    }
    let text = s.to_string();
    if text.is_empty() {
        return;
    }
    let wide = to_wide(&text);
    let payload_len = wide.len().saturating_sub(1); // exclude null terminator
    unsafe {
        OutputDebugStringW(wide.as_ptr());
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() {
            let mut written: u32 = 0;
            WriteConsoleW(
                handle,
                wide.as_ptr(),
                payload_len as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

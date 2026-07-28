use crate::config::RingType;
// ───────────────────────────────────────────────
//  WinMM FFI
// ───────────────────────────────────────────────

#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(pszSound: *const u16, hmod: *mut std::ffi::c_void, fdwSound: u32) -> i32;
    fn waveOutSetVolume(hwo: *mut std::ffi::c_void, dwVolume: u32) -> u32;
    #[allow(dead_code)]
    fn waveOutGetVolume(hwo: *mut std::ffi::c_void, pdwVolume: *mut u32) -> u32;
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
#[allow(dead_code)]
const MMSYSERR_NOERROR: u32 = 0;

// Embedded default WAVs
const START_WAV: &[u8] = include_bytes!("../res/start.wav");
const END_WAV: &[u8] = include_bytes!("../res/end.wav");
const SPECIAL_WAV: &[u8] = include_bytes!("../res/special.wav");

// ───────────────────────────────────────────────
//  Audio player
// ───────────────────────────────────────────────

pub struct AudioPlayer {
    current_volume: std::sync::Mutex<u8>, // 0-100
}

impl AudioPlayer {
    pub fn new(default_volume: u8) -> Self {
        let vol = default_volume.min(100);
        // Set initial wave volume
        Self::set_wave_volume(vol);
        AudioPlayer {
            current_volume: std::sync::Mutex::new(vol),
        }
    }

    /// Set volume (0-100). Affects subsequent plays.
    #[allow(dead_code)]
    pub fn set_volume(&self, volume: u8) {
        let vol = volume.min(100);
        *self.current_volume.lock().unwrap() = vol;
        Self::set_wave_volume(vol);
    }

    fn set_wave_volume(vol: u8) {
        // Convert 0-100 to 0-65535 (logarithmic scale is better but linear works)
        let v = ((vol as f32 / 100.0) * 65535.0) as u32;
        let dw = v | (v << 16); // left + right channels
        unsafe {
            waveOutSetVolume(std::ptr::null_mut(), dw);
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
                        PlaySoundW(
                            data.as_ptr() as *const u16,
                            std::ptr::null_mut(),
                            SND_MEMORY | SND_ASYNC,
                        );
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
            debug_log(&format!(
                "[tip_clock] 自定义音频文件不存在: {}\n",
                full_path.display()
            ));
            return;
        }

        let wide = to_wide(&full_path.to_string_lossy());
        unsafe {
            PlaySoundW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }

    /// For volume display (0-100)
    #[allow(dead_code)]
    pub fn current_volume(&self) -> u8 {
        *self.current_volume.lock().unwrap()
    }
}

// ───────────────────────────────────────────────
//  UTF-16 conversion + debug output helpers
// ───────────────────────────────────────────────

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
            WriteConsoleW(
                handle,
                wide.as_ptr(),
                (wide.len() - 1) as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

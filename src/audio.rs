use crate::config::RingType;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::io::{BufReader, Cursor};
use std::path::{Component, Path};

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

const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
const INVALID_HANDLE_VALUE: isize = -1;

const START_WAV: &[u8] = include_bytes!("../res/start.wav");
const END_WAV: &[u8] = include_bytes!("../res/end.wav");
const SPECIAL_WAV: &[u8] = include_bytes!("../res/special.wav");

/// Application-scoped audio output. Sink volume affects only this process and
/// never changes the Windows device/mixer volume.
pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer").finish_non_exhaustive()
    }
}

impl AudioPlayer {
    pub fn new(default_volume: u8) -> Result<Self, String> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to open the default audio output: {e}"))?;
        let sink = Sink::connect_new(stream.mixer());
        sink.set_volume(f32::from(default_volume.min(100)) / 100.0);
        Ok(Self {
            _stream: stream,
            sink,
        })
    }

    /// Queue a reminder sound. Multiple reminders at the same second play in
    /// schedule order rather than interrupting each other.
    pub fn play(&self, ring: RingType, custom_file: Option<&str>, exe_dir: &Path) {
        let result = match ring {
            RingType::None => Ok(()),
            RingType::Start => self.queue_embedded(START_WAV),
            RingType::End => self.queue_embedded(END_WAV),
            RingType::Special => self.queue_embedded(SPECIAL_WAV),
            RingType::Custom => custom_file
                .ok_or_else(|| "custom ring requires custom_file".to_string())
                .and_then(|name| self.queue_custom(name, exe_dir)),
        };
        if let Err(error) = result {
            debug_log(format!("[audio] {error}; using the embedded fallback\n"));
            if ring != RingType::Special {
                let _ = self.queue_embedded(SPECIAL_WAV);
            }
        }
    }

    fn queue_embedded(&self, data: &'static [u8]) -> Result<(), String> {
        let decoder = Decoder::new_wav(Cursor::new(data))
            .map_err(|e| format!("failed to decode embedded WAV: {e}"))?;
        self.sink.append(decoder);
        Ok(())
    }

    fn queue_custom(&self, filename: &str, exe_dir: &Path) -> Result<(), String> {
        let relative = safe_wav_name(filename)?;
        let full_path = exe_dir.join(relative);
        let file = std::fs::File::open(&full_path)
            .map_err(|e| format!("cannot open custom WAV '{}': {e}", full_path.display()))?;
        let decoder = Decoder::new_wav(BufReader::new(file))
            .map_err(|e| format!("cannot decode custom WAV '{}': {e}", full_path.display()))?;
        self.sink.append(decoder);
        Ok(())
    }
}

fn safe_wav_name(filename: &str) -> Result<std::path::PathBuf, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("custom_file is empty".into());
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || path.file_name() != Some(path.as_os_str())
    {
        return Err("custom_file must be a file name inside the application directory".into());
    }
    let mut result = path.to_path_buf();
    match result.extension().and_then(|ext| ext.to_str()) {
        None => {
            result.set_extension("wav");
        }
        Some(ext) if ext.eq_ignore_ascii_case("wav") => {}
        Some(_) => return Err("custom_file must have the .wav extension".into()),
    }
    Ok(result)
}

pub(crate) fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn debug_log(s: impl ToString) {
    if !cfg!(debug_assertions) {
        return;
    }
    let text = s.to_string();
    if text.is_empty() {
        return;
    }
    let wide = to_wide(&text);
    let payload_len = wide.len().saturating_sub(1).min(u32::MAX as usize) as u32;
    // SAFETY: wide is NUL-terminated and remains alive throughout both calls.
    unsafe {
        OutputDebugStringW(wide.as_ptr());
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() && handle as isize != INVALID_HANDLE_VALUE {
            let mut written = 0;
            WriteConsoleW(
                handle,
                wide.as_ptr(),
                payload_len,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_wav_is_confined_to_app_directory() {
        assert_eq!(safe_wav_name("alert").unwrap(), Path::new("alert.wav"));
        assert_eq!(safe_wav_name("alert.WAV").unwrap(), Path::new("alert.WAV"));
        assert!(safe_wav_name("../alert").is_err());
        assert!(safe_wav_name("folder/alert").is_err());
        assert!(safe_wav_name(r"C:\\alert.wav").is_err());
        assert!(safe_wav_name("alert.mp3").is_err());
    }
}

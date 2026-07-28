use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::io::BufReader;
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

/// Application-scoped audio output. Sink volume affects only this process and
/// never changes the Windows device/mixer volume.
pub struct AudioPlayer {
    _device_sink: MixerDeviceSink,
    player: Player,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer").finish_non_exhaustive()
    }
}

impl AudioPlayer {
    pub fn new(default_volume: u8) -> Result<Self, String> {
        let device_sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Failed to open the default audio output: {e}"))?;
        let player = Player::connect_new(device_sink.mixer());
        player.set_volume(f32::from(default_volume.min(100)) / 100.0);
        Ok(Self {
            _device_sink: device_sink,
            player,
        })
    }

    /// Queue an external reminder sound. Multiple reminders at the same second
    /// play in schedule order rather than interrupting one another.
    pub fn play(&self, filename: &str, audio_dir: &Path) {
        if let Err(error) = self.queue_file(filename, audio_dir) {
            debug_log(format!("[audio] {error}\n"));
        }
    }

    fn queue_file(&self, filename: &str, audio_dir: &Path) -> Result<(), String> {
        let full_path = resolve_audio_path(filename, audio_dir)?;
        let file = std::fs::File::open(&full_path)
            .map_err(|e| format!("cannot open custom audio '{}': {e}", full_path.display()))?;
        // Decoder::try_from detects the enabled WAV/FLAC/MP3 formats from the
        // stream, so a misleading extension cannot select the wrong decoder.
        let decoder = Decoder::try_from(BufReader::new(file))
            .map_err(|e| format!("cannot decode custom audio '{}': {e}", full_path.display()))?;
        self.player.append(decoder);
        Ok(())
    }
}

fn validate_audio_name(filename: &str) -> Result<&Path, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("audio is empty".into());
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || path.file_name() != Some(path.as_os_str())
    {
        return Err("audio must be a file name inside the configuration directory".into());
    }
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str())
        && !ext.eq_ignore_ascii_case("wav")
        && !ext.eq_ignore_ascii_case("flac")
        && !ext.eq_ignore_ascii_case("mp3")
    {
        return Err("audio must be a WAV, FLAC, or MP3 file".into());
    }
    Ok(path)
}

fn resolve_audio_path(filename: &str, audio_dir: &Path) -> Result<std::path::PathBuf, String> {
    let relative = validate_audio_name(filename)?;
    if relative.extension().is_some() {
        return Ok(audio_dir.join(relative));
    }

    // An omitted extension searches all supported formats deterministically.
    // WAV wins when multiple files have the same stem.
    for extension in ["wav", "flac", "mp3"] {
        let candidate = audio_dir.join(relative).with_extension(extension);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "custom audio not found: expected '{}.wav', '{}.flac', or '{}.mp3'",
        relative.display(),
        relative.display(),
        relative.display()
    ))
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
    fn custom_audio_name_is_confined_and_format_checked() {
        assert_eq!(validate_audio_name("alert").unwrap(), Path::new("alert"));
        assert_eq!(
            validate_audio_name("alert.WAV").unwrap(),
            Path::new("alert.WAV")
        );
        assert_eq!(
            validate_audio_name("alert.flac").unwrap(),
            Path::new("alert.flac")
        );
        assert_eq!(
            validate_audio_name("alert.MP3").unwrap(),
            Path::new("alert.MP3")
        );
        assert!(validate_audio_name("../alert.mp3").is_err());
        assert!(validate_audio_name("folder/alert.flac").is_err());
        assert!(validate_audio_name(r"C:\\alert.wav").is_err());
        assert!(validate_audio_name("alert.ogg").is_err());
    }

    #[test]
    fn omitted_extension_prefers_wav_then_flac_then_mp3() {
        let root =
            std::env::temp_dir().join(format!("tip-clock-audio-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("alert.mp3"), []).unwrap();
        assert_eq!(
            resolve_audio_path("alert", &root).unwrap(),
            root.join("alert.mp3")
        );
        std::fs::write(root.join("alert.flac"), []).unwrap();
        assert_eq!(
            resolve_audio_path("alert", &root).unwrap(),
            root.join("alert.flac")
        );
        std::fs::write(root.join("alert.wav"), []).unwrap();
        assert_eq!(
            resolve_audio_path("alert", &root).unwrap(),
            root.join("alert.wav")
        );
        assert_eq!(
            resolve_audio_path("alert.mp3", &root).unwrap(),
            root.join("alert.mp3")
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}

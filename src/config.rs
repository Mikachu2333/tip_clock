use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn ReplaceFileW(
        replaced: *const u16,
        replacement: *const u16,
        backup: *const u16,
        flags: u32,
        exclude: *mut std::ffi::c_void,
        reserved: *mut std::ffi::c_void,
    ) -> i32;
}

// ───────────────────────────────────────────────
//  Schedule entry
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: String,
    /// Optional WAV/FLAC/MP3 file in the configuration directory.
    /// An omitted value creates a silent visual reminder. The alias imports
    /// custom-file entries from version 1.3.x when the config is next saved.
    #[serde(default, alias = "custom_file")]
    pub audio: Option<String>,
}

// ───────────────────────────────────────────────
//  General config
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub auto_start: bool,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub bg_opacity: u8, // 0-100
    pub text_r: u8,
    pub text_g: u8,
    pub text_b: u8,
    pub display_time: u32,  // seconds, 1-60
    pub volume: u8,         // 0-100
    pub hotkey_mod: String, // e.g., "ctrl+alt"
    pub hotkey_key: String, // e.g., "T"
    pub window_x: i32,      // -1 = auto, otherwise pixel position
    pub window_y: i32,      // -1 = auto, otherwise pixel position
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            auto_start: false,
            bg_r: 255,
            bg_g: 255,
            bg_b: 255,
            bg_opacity: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
            display_time: 3,
            volume: 80,
            hotkey_mod: "Ctrl+Alt".into(),
            hotkey_key: "B".into(),
            window_x: -1,
            window_y: -1,
        }
    }
}

impl GeneralConfig {
    /// Clamp all user-editable values to valid ranges (repair broken config files).
    /// Note: u8 fields (bg_r/g/b, text_r/g/b) are validated by serde itself —
    /// values > 255 cause a parse error, so no explicit clamp is needed.
    pub fn clamp(&mut self) {
        self.bg_opacity = self.bg_opacity.min(100);
        self.display_time = self.display_time.clamp(1, 60);
        self.volume = self.volume.min(100);
        // Validate only integer corruption here. Actual monitor/work-area
        // clamping is performed by gui.rs, where negative multi-monitor
        // coordinates are valid.
        self.clamp_window_position();
    }

    /// Reset window position to 0,0 if it exceeds screen bounds.
    /// Keeps -1 (auto) as-is.
    pub fn clamp_window_position(&mut self) {
        if self.window_x == -1 && self.window_y == -1 {
            return;
        }
        // Keep ordinary negative coordinates for monitors left/above the
        // primary display; reject only implausible/corrupt values.
        if self.window_x < -1_000_000
            || self.window_x > 1_000_000
            || self.window_y < -1_000_000
            || self.window_y > 1_000_000
        {
            self.window_x = 0;
            self.window_y = 0;
        }
    }
}

// ───────────────────────────────────────────────
//  Full config
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct ParsedEntry {
    pub total_sec: u32,
    pub hour: u32,
    pub minute: u32,
    pub audio: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub schedule: Vec<ScheduleEntry>,
    pub entries: Vec<ParsedEntry>,
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
}

/// Template for auto-creating config with comments — Chinese
const CONFIG_TEMPLATE_ZH: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock 配置文件
# ──────────────────────────────────────────────
#  时间格式: HH:MM:SS (24小时制)
#  audio: 配置目录内的 WAV、FLAC 或 MP3 文件名（可省略扩展名）
#  省略 audio 时仅显示时钟，不播放音频
#  修改后需重启程序生效
#  使用 # 开头的行是注释, 不会被读取
# ──────────────────────────────────────────────

[general]
# 开机自启 (Windows 启动时自动运行)
auto_start = {auto_start}

# 背景颜色 RGB (0-255), 不透明度 (0-100, 0=全透明)
bg_r = {bg_r}
bg_g = {bg_g}
bg_b = {bg_b}
bg_opacity = {bg_opacity}

# 文字颜色 RGB (0-255)
text_r = {text_r}
text_g = {text_g}
text_b = {text_b}

# 临时显示时间, 秒 (1-60)
display_time = {display_time}

# 默认音量 (0-100)
volume = {volume}

# 显示/隐藏快捷键
# 修饰键: alt, ctrl, shift, win (可用 + 组合)
hotkey_mod = "{hotkey_mod}"
hotkey_key = "{hotkey_key}"

# 窗口显示位置 (左上角像素坐标, -1 表示自动定位)
window_x = {window_x}
window_y = {window_y}

[[schedule]]
# 有声音的提醒；首次创建配置时会同步生成 demo.mp3
time = "08:00:00"
audio = "demo"

[[schedule]]
# 静音提醒：省略 audio
time = "13:42:57"
"#;

/// Template for auto-creating config with comments — English
const CONFIG_TEMPLATE_EN: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock Configuration
# ──────────────────────────────────────────────
#  Time format: HH:MM:SS (24-hour)
#  audio: WAV, FLAC, or MP3 file in the config directory (extension optional)
#  Omit audio for a silent visual reminder
#  Changes require a program restart to take effect
#  Lines starting with # are comments
# ──────────────────────────────────────────────

[general]
# Auto-start with Windows
auto_start = {auto_start}

# Background color RGB (0-255), opacity (0-100, 0=fully transparent, 100=solid)
bg_r = {bg_r}
bg_g = {bg_g}
bg_b = {bg_b}
bg_opacity = {bg_opacity}

# Text color RGB (0-255)
text_r = {text_r}
text_g = {text_g}
text_b = {text_b}

# Display duration in seconds (1-60)
display_time = {display_time}

# Default volume (0-100)
volume = {volume}

# Show / hide hotkey
# Modifiers: alt, ctrl, shift, win (use + to combine)
hotkey_mod = "{hotkey_mod}"
hotkey_key = "{hotkey_key}"

# Window position (pixel coordinates of top-left corner, -1 = auto)
window_x = {window_x}
window_y = {window_y}

[[schedule]]
# Audible reminder; demo.mp3 is created with the initial config
time = "08:00:00"
audio = "demo"

[[schedule]]
# Silent reminder: omit audio
time = "13:42:57"
"#;

/// Select the appropriate config template based on system language
fn config_template() -> &'static str {
    match crate::i18n::lang() {
        crate::i18n::Lang::Zh => CONFIG_TEMPLATE_ZH,
        crate::i18n::Lang::En => CONFIG_TEMPLATE_EN,
    }
}

fn choose_config_path(exe_path: &Path) -> Result<PathBuf, String> {
    // Prefer portable configuration whenever the EXE directory is writable,
    // including when config.toml already exists but is read-only.
    let probe = exe_path.with_extension(format!("write-test-{}.tmp", std::process::id()));
    let writable = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .and_then(|_| std::fs::remove_file(&probe))
        .is_ok();
    if writable {
        return Ok(exe_path.to_path_buf());
    }

    let local = std::env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("EXE directory is not writable and LOCALAPPDATA is unavailable")?
        .join("TipClock");
    std::fs::create_dir_all(&local).map_err(|e| {
        format!(
            "Failed to create config directory '{}': {e}",
            local.display()
        )
    })?;
    let fallback = local.join("config.toml");
    // Preserve an existing portable configuration on the first fallback.
    if exe_path.exists() && !fallback.exists() {
        let data = std::fs::read(exe_path)
            .map_err(|e| format!("Failed to migrate existing config: {e}"))?;
        atomic_write(&fallback, &data)?;
    }
    Ok(fallback)
}

fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_owned()).to_string()
}

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    const MAX_REPLACE_ATTEMPTS: u32 = 6;
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_SHARING_VIOLATION: i32 = 32;
    const ERROR_LOCK_VIOLATION: i32 = 33;

    let parent = path.parent().ok_or("Config path has no parent directory")?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create config directory: {e}"))?;

    // A per-process sequence prevents stale files or two different target
    // names from reusing the same temporary path.
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Config file name is not valid UTF-8")?;
    let temp = parent.join(format!(
        ".{file_name}.tmp-{}-{sequence}",
        std::process::id()
    ));

    let result = (|| -> std::io::Result<()> {
        // Keep the File in a nested scope. sync_all flushes data but does not
        // close the Windows handle; ReplaceFileW requires the replacement file
        // to be closed before it can rename it.
        {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?;
            file.write_all(data)?;
            file.sync_all()?;
        }

        if !path.exists() {
            return std::fs::rename(&temp, path);
        }

        let destination = crate::audio::to_wide(&path.to_string_lossy());
        let replacement = crate::audio::to_wide(&temp.to_string_lossy());
        let mut delay_ms = 15u64;
        for attempt in 1..=MAX_REPLACE_ATTEMPTS {
            // SAFETY: both UTF-16 paths are NUL-terminated and remain alive
            // during this synchronous call. The replacement file handle was
            // closed above; no backup/exclusion buffers are used.
            if unsafe {
                ReplaceFileW(
                    destination.as_ptr(),
                    replacement.as_ptr(),
                    std::ptr::null(),
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            } != 0
            {
                return Ok(());
            }

            let error = std::io::Error::last_os_error();
            let retryable = matches!(
                error.raw_os_error(),
                Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION)
            );
            if !retryable || attempt == MAX_REPLACE_ATTEMPTS {
                return Err(error);
            }
            std::thread::sleep(Duration::from_millis(delay_ms));
            delay_ms *= 2;
        }
        unreachable!("replace loop always returns")
    })();

    if result.is_err() {
        // Best effort only: retain the original error, which is more useful
        // than a secondary cleanup failure. Unique names avoid later clashes.
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(|e| {
        format!(
            "Failed to atomically write config '{}': {e}",
            path.display()
        )
    })
}

pub fn default_schedule() -> Vec<ScheduleEntry> {
    vec![
        ScheduleEntry {
            time: "08:00:00".into(),
            audio: Some("demo".into()),
        },
        ScheduleEntry {
            time: "12:00:00".into(),
            audio: None,
        },
    ]
}

const DEMO_AUDIO: &[u8] = include_bytes!("../res/demo.mp3");

fn install_demo_audio(config_dir: &Path) -> Result<(), String> {
    let demo_path = config_dir.join("demo.mp3");
    if !demo_path.exists() {
        atomic_write(&demo_path, DEMO_AUDIO)?;
    }
    Ok(())
}

impl Config {
    pub fn load_or_create() -> Result<Self, String> {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get EXE path: {e}"))?;
        let exe_dir = exe_path
            .parent()
            .ok_or("Failed to get EXE directory")?
            .to_path_buf();
        let exe_config_path = exe_dir.join("config.toml");
        let config_path = choose_config_path(&exe_config_path)?;

        if config_path.exists() {
            let mut cfg = Self::load_and_merge(&config_path)?;
            cfg.general.clamp();
            let entries = Config::build_entries(&cfg.schedule);
            let config_dir = config_path
                .parent()
                .ok_or("Config path has no parent directory")?
                .to_path_buf();
            Ok(Config {
                general: cfg.general,
                schedule: cfg.schedule,
                entries,
                config_path: config_path.clone(),
                config_dir,
            })
        } else {
            // Create default config from template
            let default = GeneralConfig::default();
            let content = config_template()
                .replace("{auto_start}", &default.auto_start.to_string())
                .replace("{bg_r}", &default.bg_r.to_string())
                .replace("{bg_g}", &default.bg_g.to_string())
                .replace("{bg_b}", &default.bg_b.to_string())
                .replace("{bg_opacity}", &default.bg_opacity.to_string())
                .replace("{text_r}", &default.text_r.to_string())
                .replace("{text_g}", &default.text_g.to_string())
                .replace("{text_b}", &default.text_b.to_string())
                .replace("{display_time}", &default.display_time.to_string())
                .replace("{volume}", &default.volume.to_string())
                .replace("{hotkey_mod}", &default.hotkey_mod)
                .replace("{hotkey_key}", &default.hotkey_key)
                .replace("{window_x}", &default.window_x.to_string())
                .replace("{window_y}", &default.window_y.to_string());
            let config_dir = config_path
                .parent()
                .ok_or("Config path has no parent directory")?
                .to_path_buf();
            // Install the demo first. If config creation then fails, the next
            // startup still sees no config and safely retries without
            // overwriting an existing demo file.
            install_demo_audio(&config_dir)?;
            atomic_write(&config_path, content.as_bytes())?;

            let schedule = default_schedule();
            let entries = Config::build_entries(&schedule);
            Ok(Config {
                general: default,
                schedule,
                entries,
                config_path,
                config_dir,
            })
        }
    }

    /// Load config from file, merge with defaults for any missing keys
    fn load_and_merge(path: &PathBuf) -> Result<ConfigFile, String> {
        let raw =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;

        // Normalize common CJK punctuation in the entire file
        let raw_cleaned = raw
            .replace('：', ":") // full-width colon
            .replace('＝', "=") // full-width equals
            .replace('，', ",") // full-width comma
            .replace('；', ";") // full-width semicolon
            .replace('＃', "#") // full-width hash (comment)
            .replace('【', "[") // full-width bracket
            .replace('】', "]")
            .replace('。', ".")
            .replace('　', " "); // full-width space

        // Auto-correct bare time values: `time = 09:00` → `time = "09:00"`
        let raw_cleaned = auto_quote_bare_times(&raw_cleaned);

        // Parse cleaned TOML text
        let cfg: ConfigFile =
            toml::from_str(&raw_cleaned).map_err(|e| format!("Failed to parse config: {e}"))?;

        // Validate schedule entries
        let mut valid_schedule: Vec<ScheduleEntry> = Vec::new();
        for entry in &cfg.schedule {
            if let Some(normalized) = normalize_time(&entry.time)
                && let Some((h, m, s)) = parse_normalized(&normalized)
                && h < 24
                && m < 60
                && s < 60
            {
                let mut corrected = entry.clone();
                if corrected.time != normalized {
                    crate::audio::debug_log(format!(
                        "[tip_clock] auto-corrected: '{}' -> '{}'\n",
                        entry.time, normalized
                    ));
                }
                corrected.time = normalized;
                valid_schedule.push(corrected);
                continue;
            }
            crate::audio::debug_log(format!(
                "[tip_clock] ignored invalid time: {}\n",
                entry.time
            ));
        }

        // An explicitly empty schedule is valid. If entries were supplied but
        // every one is invalid, fail visibly instead of silently scheduling
        // unrelated default reminders.
        if valid_schedule.is_empty() && !cfg.schedule.is_empty() {
            return Err("All configured schedule entries are invalid".into());
        }
        let final_schedule = valid_schedule;

        let merged = ConfigFile {
            general: cfg.general,
            schedule: final_schedule,
        };

        Ok(merged)
    }

    fn build_entries(schedule: &[ScheduleEntry]) -> Vec<ParsedEntry> {
        let mut entries: Vec<ParsedEntry> = schedule
            .iter()
            .filter_map(|entry| {
                let (h, m, s) = parse_hhmmss(&entry.time)?;
                if h >= 24 || m >= 60 || s >= 60 {
                    return None;
                }
                Some(ParsedEntry {
                    total_sec: h * 3600 + m * 60 + s,
                    hour: h,
                    minute: m,
                    audio: entry.audio.clone(),
                })
            })
            .collect();
        entries.sort_by_key(|e| e.total_sec);
        entries
    }

    /// Find entries in `(start, end]`. This prevents a short message-loop
    /// stall from losing a reminder whose exact second was not sampled.
    pub fn entries_between(&self, start: u32, end: u32) -> Vec<&ParsedEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.total_sec > start && entry.total_sec <= end)
            .collect()
    }
}

/// Normalize and parse a time string, auto-correcting common mistakes.
///
/// Supported inputs (all produce the same canonical "HH:MM:SS" output):
///   "9"           → "09:00:00"
///   "09"          → "09:00:00"
///   "9:00"        → "09:00:00"
///   "09:00"       → "09:00:00"
///   "9:00:00"     → "09:00:00"
///   "9:0:0"       → "09:00:00"
///   "090000"      → "09:00:00"  (compact 6-digit)
///   "0900"        → "09:00:00"  (compact 4-digit)
///
/// Also auto-corrects:  full-width colon "：" → ":"  and Chinese comma "，" → ","
pub fn normalize_time(raw: &str) -> Option<String> {
    // Step 1: replace full-width punctuation
    let s = raw
        .replace('：', ":")
        .replace('，', ",")
        .replace('　', " ") // full-width space
        .replace('\u{200B}', "") // zero-width space
        .trim()
        .to_string();

    if s.is_empty() {
        return None;
    }

    // Step 2: try compact digit string (no separators)
    if !s.contains(':') && s.chars().all(|c| c.is_ascii_digit()) {
        return match s.len() {
            1..=2 => {
                // "9" or "09" → hours only
                let h: u32 = s.parse().ok()?;
                (h < 24).then(|| format!("{h:02}:00:00"))
            }
            3..=4 => {
                // 3 digits: 1h+2m ("930" → 9:30).  4 digits: 2h+2m ("0930" → 09:30).
                let split = if s.len() == 3 { 1 } else { 2 };
                let h: u32 = s[..split].parse().ok()?;
                let m: u32 = s[split..].parse().ok()?;
                (h < 24 && m < 60).then(|| format!("{h:02}:{m:02}:00"))
            }
            5..=6 => {
                // "093000" or "93000" → HHMMSS
                let padded = format!("{:0>6}", s);
                let h: u32 = padded[0..2].parse().ok()?;
                let m: u32 = padded[2..4].parse().ok()?;
                let sec: u32 = padded[4..6].parse().ok()?;
                (h < 24 && m < 60 && sec < 60).then(|| format!("{h:02}:{m:02}:{sec:02}"))
            }
            _ => None,
        };
    }

    // Step 3: split by colon (1-3 parts)
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    match parts.len() {
        1 => {
            let h: u32 = parts[0].parse().ok()?;
            (h < 24).then(|| format!("{h:02}:00:00"))
        }
        2 => {
            let h: u32 = parts[0].parse().ok()?;
            let m: u32 = parts[1].parse().ok()?;
            (h < 24 && m < 60).then(|| format!("{h:02}:{m:02}:00"))
        }
        3 => {
            let h: u32 = parts[0].parse().ok()?;
            let m: u32 = parts[1].parse().ok()?;
            let sec: u32 = parts[2].parse().ok()?;
            (h < 24 && m < 60 && sec < 60).then(|| format!("{h:02}:{m:02}:{sec:02}"))
        }
        _ => None,
    }
}

/// Internal: parse a normalized time string into (h, m, s) components.
/// Assumes input was already passed through `normalize_time`.
fn parse_normalized(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.splitn(3, ':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let sec: u32 = parts.next().unwrap_or("0").parse().ok()?;
    (h < 24 && m < 60 && sec < 60).then_some((h, m, sec))
}

/// Parse "HH:MM:SS" or "HH:MM" → (h, m, s)
/// (Legacy, kept for backward-compat; prefer `normalize_time` + `parse_normalized`.)
pub fn parse_hhmmss(s: &str) -> Option<(u32, u32, u32)> {
    // Delegate to normalize_time for auto-correction
    let normalized = normalize_time(s)?;
    parse_normalized(&normalized)
}

/// Auto-wrap bare time values in quotes for TOML compatibility.
/// Converts `time = 09:00` → `time = "09:00"`.
fn auto_quote_bare_times(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("time") {
            // Find the `=` sign
            if let Some(eq_pos) = trimmed.find('=') {
                let after_eq = trimmed[eq_pos + 1..].trim();
                // Already quoted → keep as-is
                if after_eq.starts_with('"') || after_eq.starts_with('\'') {
                    result.push_str(line);
                } else if !after_eq.is_empty() && after_eq.contains(':') && !after_eq.contains(' ')
                {
                    // Bare time value like `09:00` or `9:00:00`
                    let indent = &line[..line.len() - trimmed.len()];
                    let before_eq = &trimmed[..=eq_pos];
                    result.push_str(indent);
                    result.push_str(before_eq);
                    result.push(' ');
                    result.push('"');
                    result.push_str(after_eq);
                    result.push('"');
                } else {
                    result.push_str(line);
                }
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    result
}

/// Write back the config using the template with current values
impl Config {
    pub fn save_to_file(&self) -> Result<(), String> {
        // Build the [general] section from the template to preserve comments.
        let content = config_template()
            .replace("{auto_start}", &self.general.auto_start.to_string())
            .replace("{bg_r}", &self.general.bg_r.to_string())
            .replace("{bg_g}", &self.general.bg_g.to_string())
            .replace("{bg_b}", &self.general.bg_b.to_string())
            .replace("{bg_opacity}", &self.general.bg_opacity.to_string())
            .replace("{text_r}", &self.general.text_r.to_string())
            .replace("{text_g}", &self.general.text_g.to_string())
            .replace("{text_b}", &self.general.text_b.to_string())
            .replace("{display_time}", &self.general.display_time.to_string())
            .replace("{volume}", &self.general.volume.to_string())
            .replace("{hotkey_mod}", &self.general.hotkey_mod)
            .replace("{hotkey_key}", &self.general.hotkey_key)
            .replace("{window_x}", &self.general.window_x.to_string())
            .replace("{window_y}", &self.general.window_y.to_string());

        // Keep only the [general] section header and everything above the first
        // [[schedule]] line.  Then append the *actual* schedule entries so that
        // user-added / user-modified times are preserved across saves.
        let general_section = content
            .split_once("[[schedule]]")
            .map(|(head, _)| head)
            .unwrap_or(&content);

        let mut out = general_section.trim_end().to_string();
        out.push_str("\n\n");
        for entry in &self.schedule {
            out.push_str("[[schedule]]\n");
            out.push_str(&format!("time = {}\n", toml_string(&entry.time)));
            if let Some(ref audio) = entry.audio {
                out.push_str(&format!("audio = {}\n", toml_string(audio)));
            }
            out.push('\n');
        }

        atomic_write(&self.config_path, out.as_bytes())
    }
}

// ───────────────────────────────────────────────
//  Unit tests
// ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_time_variants() {
        assert_eq!(normalize_time("9"), Some("09:00:00".into()));
        assert_eq!(normalize_time("09"), Some("09:00:00".into()));
        assert_eq!(normalize_time("9:00"), Some("09:00:00".into()));
        assert_eq!(normalize_time("09:00"), Some("09:00:00".into()));
        assert_eq!(normalize_time("9:00:00"), Some("09:00:00".into()));
        assert_eq!(normalize_time("9:0:0"), Some("09:00:00".into()));
    }

    #[test]
    fn test_normalize_time_compact() {
        assert_eq!(normalize_time("090000"), Some("09:00:00".into()));
        assert_eq!(normalize_time("90000"), Some("09:00:00".into()));
        assert_eq!(normalize_time("0930"), Some("09:30:00".into()));
        assert_eq!(normalize_time("930"), Some("09:30:00".into()));
    }

    #[test]
    fn test_normalize_time_fullwidth() {
        assert_eq!(normalize_time("9：00"), Some("09:00:00".into()));
        assert_eq!(normalize_time("9：00：00"), Some("09:00:00".into()));
    }

    #[test]
    fn test_normalize_time_whitespace() {
        assert_eq!(normalize_time(" 9:00 "), Some("09:00:00".into()));
        assert_eq!(normalize_time("09：00　"), Some("09:00:00".into()));
    }

    #[test]
    fn test_normalize_time_invalid() {
        assert_eq!(normalize_time("25:00"), None);
        assert_eq!(normalize_time("12:60"), None);
        assert_eq!(normalize_time("12:00:60"), None);
        assert_eq!(normalize_time("abc"), None);
        assert_eq!(normalize_time(""), None);
    }

    #[test]
    fn test_auto_quote_bare_times() {
        let input = "[[schedule]]\ntime = 09:00\naudio = \"demo\"\n";
        let expect = "[[schedule]]\ntime = \"09:00\"\naudio = \"demo\"\n";
        assert_eq!(auto_quote_bare_times(input), expect);
    }

    #[test]
    fn test_auto_quote_already_quoted() {
        let input = "time = \"09:00\"\n";
        assert_eq!(auto_quote_bare_times(input), input);
    }

    #[test]
    fn test_parse_hhmmss_delegates() {
        assert_eq!(parse_hhmmss("9:30"), Some((9, 30, 0)));
        assert_eq!(parse_hhmmss("09:30:00"), Some((9, 30, 0)));
        assert_eq!(parse_hhmmss("9：30：00"), Some((9, 30, 0)));
    }

    #[test]
    fn test_normalize_time_edge_cases() {
        // Zero values
        assert_eq!(normalize_time("0"), Some("00:00:00".into()));
        assert_eq!(normalize_time("00:00"), Some("00:00:00".into()));
        assert_eq!(normalize_time("0:0:0"), Some("00:00:00".into()));
        // Max valid values
        assert_eq!(normalize_time("23:59:59"), Some("23:59:59".into()));
        // 24:00 boundary — invalid hour
        assert_eq!(normalize_time("24:00"), None);
    }

    #[test]
    fn test_demo_audio_install_is_non_destructive() {
        let root =
            std::env::temp_dir().join(format!("tip-clock-config-demo-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        install_demo_audio(&root).unwrap();
        assert_eq!(std::fs::read(root.join("demo.mp3")).unwrap(), DEMO_AUDIO);

        std::fs::write(root.join("demo.mp3"), b"user audio").unwrap();
        install_demo_audio(&root).unwrap();
        assert_eq!(std::fs::read(root.join("demo.mp3")).unwrap(), b"user audio");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_entries_at_single() {
        let entries = vec![
            ScheduleEntry {
                time: "09:00:00".into(),
                audio: Some("demo".into()),
            },
            ScheduleEntry {
                time: "12:00:00".into(),
                audio: None,
            },
        ];
        let parsed = Config::build_entries(&entries);
        assert_eq!(parsed.len(), 2);
        // Simulate entries_at logic
        let at_9 = parsed
            .iter()
            .filter(|e| e.total_sec > 9 * 3600 - 1 && e.total_sec <= 9 * 3600)
            .collect::<Vec<_>>();
        assert_eq!(at_9.len(), 1);
        assert_eq!(at_9[0].audio.as_deref(), Some("demo"));
        // No match
        let at_10 = parsed
            .iter()
            .filter(|e| e.total_sec == 10 * 3600)
            .collect::<Vec<_>>();
        assert!(at_10.is_empty());
    }

    #[test]
    fn test_entries_at_sorted() {
        let entries = vec![
            ScheduleEntry {
                time: "14:00:00".into(),
                audio: None,
            },
            ScheduleEntry {
                time: "08:00:00".into(),
                audio: Some("demo.mp3".into()),
            },
        ];
        let parsed = Config::build_entries(&entries);
        assert_eq!(parsed[0].hour, 8);
        assert_eq!(parsed[1].hour, 14);
    }

    #[test]
    fn test_entries_at_invalid_time_ignored() {
        let entries = vec![
            ScheduleEntry {
                time: "08:00:00".into(),
                audio: Some("demo".into()),
            },
            ScheduleEntry {
                time: "25:00:00".into(),
                audio: None,
            },
        ];
        let parsed = Config::build_entries(&entries);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].hour, 8);
    }
}

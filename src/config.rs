use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ───────────────────────────────────────────────
//  Ring type — now includes Custom and None
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RingType {
    Start,
    End,
    Special,
    Custom,
    None,
}

impl RingType {
    pub fn display_name(self) -> &'static str {
        match self {
            RingType::Start => "start",
            RingType::End => "end",
            RingType::Special => "special",
            RingType::Custom => "custom",
            RingType::None => "none",
        }
    }
}

// ───────────────────────────────────────────────
//  Schedule entry
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: String,
    pub ring: RingType,
    /// For RingType::Custom, the WAV file name (relative to EXE dir), without extension
    #[serde(default)]
    pub custom_file: Option<String>,
}

// ───────────────────────────────────────────────
//  General config
// ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            hotkey_mod: "Win+Alt".into(),
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
        // Validate window position — reset to 0,0 if out of screen bounds
        self.clamp_window_position();
    }

    /// Reset window position to 0,0 if it exceeds screen bounds.
    /// Keeps -1 (auto) as-is.
    pub fn clamp_window_position(&mut self) {
        if self.window_x == -1 && self.window_y == -1 {
            return;
        }
        // Sanity-check: negative values (other than -1) or absurdly large values
        // indicate a corrupted config — reset to 0,0.
        // Screen-bound validation happens at window creation time in gui.rs.
        if self.window_x < -1
            || self.window_x > 16384
            || self.window_y < -1
            || self.window_y > 16384
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
    pub ring: RingType,
    pub custom_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    #[allow(dead_code)]
    pub schedule: Vec<ScheduleEntry>,
    pub entries: Vec<ParsedEntry>,
    pub config_path: PathBuf,
    pub exe_dir: PathBuf,
}

/// Template for auto-creating config with comments — Chinese
const CONFIG_TEMPLATE_ZH: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock 配置文件
# ──────────────────────────────────────────────
#  时间格式: HH:MM:SS (24小时制)
#  提示音类型: start, end, special, custom, none
#    custom = 播放同目录下的 wav 文件
#    none   = 不播放音频
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
# 提醒时间与提示音
time = "08:00:00"
ring = "start"

[[schedule]]
time = "08:45:00"
ring = "end"

[[schedule]]
time = "09:40:00"
ring = "special"

[[schedule]]
time = "10:00:00"
ring = "none"

[[schedule]]
# 自定义提示音示例, 需在同目录下放置 lunch.wav 文件，仅支持 wav
time = "12:00:00"
ring = "custom"
custom_file = "lunch"
"#;

/// Template for auto-creating config with comments — English
const CONFIG_TEMPLATE_EN: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock Configuration
# ──────────────────────────────────────────────
#  Time format: HH:MM:SS (24-hour)
#  Ring types: start, end, special, custom, none
#    custom = play a .wav file from the same folder
#    none   = no sound
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
# Reminder time and ring
time = "08:00:00"
ring = "start"

[[schedule]]
time = "08:45:00"
ring = "end"

[[schedule]]
time = "09:40:00"
ring = "special"

[[schedule]]
time = "10:00:00"
ring = "none"

[[schedule]]
# custom ring example, requires lunch.wav in the same folder (only wav supported)
time = "12:00:00"
ring = "custom"
custom_file = "lunch"
"#;

/// Select the appropriate config template based on system language
fn config_template() -> &'static str {
    match crate::i18n::lang() {
        crate::i18n::Lang::Zh => CONFIG_TEMPLATE_ZH,
        crate::i18n::Lang::En => CONFIG_TEMPLATE_EN,
    }
}

pub fn default_schedule() -> Vec<ScheduleEntry> {
    vec![
        ScheduleEntry {
            time: "08:00:00".into(),
            ring: RingType::Start,
            custom_file: None,
        },
        ScheduleEntry {
            time: "08:45:00".into(),
            ring: RingType::End,
            custom_file: None,
        },
        ScheduleEntry {
            time: "09:40:00".into(),
            ring: RingType::Special,
            custom_file: None,
        },
    ]
}

impl Config {
    pub fn load_or_create() -> Result<Self, String> {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get EXE path: {e}"))?;
        let exe_dir = exe_path
            .parent()
            .ok_or("Failed to get EXE directory")?
            .to_path_buf();
        let config_path = exe_dir.join("config.toml");

        if config_path.exists() {
            let mut cfg = Self::load_and_merge(&config_path)?;
            cfg.general.clamp();
            let entries = Config::build_entries(&cfg.schedule);
            Ok(Config {
                general: cfg.general,
                schedule: cfg.schedule,
                entries,
                config_path: config_path.clone(),
                exe_dir,
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
            std::fs::write(&config_path, content)
                .map_err(|e| format!("Failed to create config file: {e}"))?;

            let schedule = default_schedule();
            let entries = Config::build_entries(&schedule);
            Ok(Config {
                general: default,
                schedule,
                entries,
                config_path,
                exe_dir,
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

        // If the user-provided schedule is entirely invalid, fall back to
        // defaults rather than running with an empty list silently.
        let final_schedule = if valid_schedule.is_empty() {
            if !cfg.schedule.is_empty() {
                crate::audio::debug_log(
                    "[tip_clock] all schedule entries invalid, using defaults\n",
                );
            }
            default_schedule()
        } else {
            valid_schedule
        };

        let merged = ConfigFile {
            general: cfg.general,
            schedule: final_schedule,
        };

        Ok(merged)
    }

    pub fn next_reminder(&self, current_h: u32, current_m: u32) -> Option<(u32, u32, RingType)> {
        let current_total = current_h * 60 + current_m;
        if let Some(e) = self
            .entries
            .iter()
            .find(|e| e.total_sec / 60 > current_total)
        {
            return Some((e.hour, e.minute, e.ring));
        }
        self.entries.first().map(|e| (e.hour, e.minute, e.ring))
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
                    ring: entry.ring,
                    custom_file: entry.custom_file.clone(),
                })
            })
            .collect();
        entries.sort_by_key(|e| e.total_sec);
        entries
    }

    /// Find all schedule entries that match the given time (HH:MM:SS)
    pub fn entries_at(&self, h: u32, m: u32, s: u32) -> Vec<&ParsedEntry> {
        let total = h * 3600 + m * 60 + s;
        self.entries
            .iter()
            .filter(|e| e.total_sec == total)
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
            out.push_str(&format!("time = \"{}\"\n", entry.time));
            out.push_str(&format!("ring = \"{}\"\n", entry.ring.display_name()));
            if let Some(ref cf) = entry.custom_file {
                out.push_str(&format!("custom_file = \"{}\"\n", cf));
            }
            out.push('\n');
        }

        std::fs::write(&self.config_path, out)
            .map_err(|e| format!("Failed to write config: {e}"))?;
        Ok(())
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
        let input = "[[schedule]]\ntime = 09:00\nring = \"start\"\n";
        let expect = "[[schedule]]\ntime = \"09:00\"\nring = \"start\"\n";
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
    fn test_entries_at_single() {
        let entries = vec![
            ScheduleEntry {
                time: "09:00:00".into(),
                ring: RingType::Start,
                custom_file: None,
            },
            ScheduleEntry {
                time: "12:00:00".into(),
                ring: RingType::End,
                custom_file: None,
            },
        ];
        let parsed = Config::build_entries(&entries);
        assert_eq!(parsed.len(), 2);
        // Simulate entries_at logic
        let at_9 = parsed
            .iter()
            .filter(|e| e.total_sec == 9 * 3600)
            .collect::<Vec<_>>();
        assert_eq!(at_9.len(), 1);
        assert_eq!(at_9[0].ring, RingType::Start);
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
                ring: RingType::End,
                custom_file: None,
            },
            ScheduleEntry {
                time: "08:00:00".into(),
                ring: RingType::Start,
                custom_file: None,
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
                ring: RingType::Start,
                custom_file: None,
            },
            ScheduleEntry {
                time: "25:00:00".into(),
                ring: RingType::End,
                custom_file: None,
            },
        ];
        let parsed = Config::build_entries(&entries);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].hour, 8);
    }
}

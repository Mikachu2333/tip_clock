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
    #[serde(default = "default_font_name")]
    pub font_name: String,
    #[serde(default = "default_font_size")]
    pub font_size: i32,
}

fn default_font_name() -> String {
    "微软雅黑".into()
}
fn default_font_size() -> i32 {
    16
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            auto_start: false,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            bg_opacity: 80,
            text_r: 255,
            text_g: 255,
            text_b: 255,
            display_time: 5,
            volume: 80,
            hotkey_mod: "ctrl+alt".into(),
            hotkey_key: "T".into(),
            font_name: default_font_name(),
            font_size: default_font_size(),
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
    pub total_sec: u32, // seconds since midnight
    pub hour: u32,
    pub minute: u32,
    #[allow(dead_code)]
    pub second: u32,
    pub ring: RingType,
    pub custom_file: Option<String>,
    #[allow(dead_code)]
    pub volume: u8,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub schedule: Vec<ScheduleEntry>,
    pub entries: Vec<ParsedEntry>,
    pub config_path: PathBuf,
    pub exe_dir: PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

/// Template for auto-creating config with comments — Chinese
const CONFIG_TEMPLATE_ZH: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock 配置文件
# ──────────────────────────────────────────────
#  时间格式: HH:MM:SS (24小时制)
#  提示音类型: start, end, special, custom, none
#    custom = 播放同目录下的 wav 文件
#    none   = 不播放音频
#  修改后自动生效 (无需重启)
#  使用 # 开头的行是注释, 不会被读取
# ──────────────────────────────────────────────

[general]
# 开机自启 (Windows 启动时自动运行)
auto_start = {auto_start}

# 背景颜色 RGB (0-255), 透明度 (0-100)
bg_r = {bg_r}
bg_g = {bg_g}
bg_b = {bg_b}
bg_opacity = {bg_opacity}

# ▸▸▸ 以下字体设置请通过托盘菜单修改，不要手动编辑 ▸▸▸
# 文字颜色 RGB (0-255)
text_r = {text_r}
text_g = {text_g}
text_b = {text_b}
# 字体名称 (通过 "字体..." 菜单选择)
font_name = "{font_name}"
# 字体大小
font_size = {font_size}
# ◂◂◂ 以上字体设置请通过托盘菜单修改，不要手动编辑 ◂◂◂

# 临时显示时间, 秒 (1-60)
display_time = {display_time}

# 默认音量 (0-100)
volume = {volume}

# 显示/隐藏快捷键
# 修饰键: alt, ctrl, shift, win (可用 + 组合)
hotkey_mod = "{hotkey_mod}"
hotkey_key = "{hotkey_key}"

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
"#;

/// Template for auto-creating config with comments — English
const CONFIG_TEMPLATE_EN: &str = r#"# ──────────────────────────────────────────────
#  Tip Clock Configuration
# ──────────────────────────────────────────────
#  Time format: HH:MM:SS (24-hour)
#  Ring types: start, end, special, custom, none
#    custom = play a .wav file from the same folder
#    none   = no sound
#  Changes take effect automatically (no restart)
#  Lines starting with # are comments
# ──────────────────────────────────────────────

[general]
# Auto-start with Windows
auto_start = {auto_start}

# Background color RGB (0-255), opacity (0-100)
bg_r = {bg_r}
bg_g = {bg_g}
bg_b = {bg_b}
bg_opacity = {bg_opacity}

# ▸▸▸ Font settings below — use tray menu, do NOT edit by hand ▸▸▸
# Text color RGB (0-255)
text_r = {text_r}
text_g = {text_g}
text_b = {text_b}
# Font name (set via "Font..." tray menu)
font_name = "{font_name}"
# Font size
font_size = {font_size}
# ◂◂◂ Font settings above — use tray menu, do NOT edit by hand ◂◂◂

# Display duration in seconds (1-60)
display_time = {display_time}

# Default volume (0-100)
volume = {volume}

# Show / hide hotkey
# Modifiers: alt, ctrl, shift, win (use + to combine)
hotkey_mod = "{hotkey_mod}"
hotkey_key = "{hotkey_key}"

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
        let exe_path = std::env::current_exe().map_err(|e| format!("获取 EXE 路径失败: {e}"))?;
        let exe_dir = exe_path
            .parent()
            .ok_or("无法获取 EXE 所在目录")?
            .to_path_buf();
        let config_path = exe_dir.join("config.toml");

        if config_path.exists() {
            let (cfg, _source) = Self::load_and_merge(&config_path)?;
            let last_modified = std::fs::metadata(&config_path)
                .ok()
                .and_then(|m| m.modified().ok());
            let entries = Config::build_entries(&cfg.schedule, cfg.general.volume);
            Ok(Config {
                general: cfg.general,
                schedule: cfg.schedule,
                entries,
                config_path: config_path.clone(),
                exe_dir,
                last_modified,
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
                .replace("{font_name}", &default.font_name)
                .replace("{font_size}", &default.font_size.to_string())
                .replace("{display_time}", &default.display_time.to_string())
                .replace("{volume}", &default.volume.to_string())
                .replace("{hotkey_mod}", &default.hotkey_mod)
                .replace("{hotkey_key}", &default.hotkey_key);
            std::fs::write(&config_path, content).map_err(|e| format!("创建配置文件失败: {e}"))?;

            let schedule = default_schedule();
            let entries = Config::build_entries(&schedule, default.volume);
            let last_modified = std::fs::metadata(&config_path)
                .ok()
                .and_then(|m| m.modified().ok());
            Ok(Config {
                general: default,
                schedule,
                entries,
                config_path,
                exe_dir,
                last_modified,
            })
        }
    }

    /// Load config from file, merge with defaults for any missing keys
    fn load_and_merge(path: &PathBuf) -> Result<(ConfigFile, String), String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {e}"))?;

        // Normalize common CJK punctuation in the entire file
        let raw_cleaned = raw
            .replace('：', ":") // full-width colon
            .replace('＝', "=") // full-width equals
            .replace('，', ",") // full-width comma
            .replace('；', ";") // full-width semicolon
            .replace('＃', "#") // full-width hash (comment)
            .replace('【', "[") // full-width bracket
            .replace('】', "]")
            .replace('　', " "); // full-width space

        // Auto-correct bare time values: `time = 09:00` → `time = "09:00"`
        let raw_cleaned = auto_quote_bare_times(&raw_cleaned);

        // Parse cleaned TOML text
        let cfg: ConfigFile =
            toml::from_str(&raw_cleaned).map_err(|e| format!("解析配置文件失败: {e}"))?;

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
                    crate::audio::debug_log(&format!(
                        "[tip_clock] 自动纠错: '{}' → '{}'\n",
                        entry.time, normalized
                    ));
                }
                corrected.time = normalized;
                valid_schedule.push(corrected);
                continue;
            }
            crate::audio::debug_log(&format!("[tip_clock] 忽略无效时间: {}\n", entry.time));
        }

        let merged = ConfigFile {
            general: cfg.general,
            schedule: if valid_schedule.is_empty() {
                default_schedule()
            } else {
                valid_schedule
            },
        };

        Ok((merged, raw))
    }

    /// Try to hot-reload the config if the file changed
    pub fn try_reload(&mut self) -> bool {
        let modified = match std::fs::metadata(&self.config_path)
            .ok()
            .and_then(|m| m.modified().ok())
        {
            Some(t) => t,
            None => return false,
        };

        if self.last_modified == Some(modified) {
            return false;
        }

        self.last_modified = Some(modified);

        match Self::load_and_merge(&self.config_path) {
            Ok((cfg, _)) => {
                self.general = cfg.general;
                self.schedule = cfg.schedule;
                self.entries = Config::build_entries(&self.schedule, self.general.volume);
                crate::audio::debug_log("[tip_clock] 配置文件已重新加载\n");
                true
            }
            Err(_e) => {
                // Keep old config on error
                false
            }
        }
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

    fn build_entries(schedule: &[ScheduleEntry], default_volume: u8) -> Vec<ParsedEntry> {
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
                    second: s,
                    ring: entry.ring,
                    custom_file: entry.custom_file.clone(),
                    volume: default_volume,
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

/// Parse "HH:MM" only (for tray tooltip)
#[allow(dead_code)]
pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
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
    #[allow(dead_code)]
    pub fn save_to_file(&self) -> Result<(), String> {
        let content = config_template()
            .replace("{auto_start}", &self.general.auto_start.to_string())
            .replace("{bg_r}", &self.general.bg_r.to_string())
            .replace("{bg_g}", &self.general.bg_g.to_string())
            .replace("{bg_b}", &self.general.bg_b.to_string())
            .replace("{bg_opacity}", &self.general.bg_opacity.to_string())
            .replace("{text_r}", &self.general.text_r.to_string())
            .replace("{text_g}", &self.general.text_g.to_string())
            .replace("{text_b}", &self.general.text_b.to_string())
            .replace("{font_name}", &self.general.font_name)
            .replace("{font_size}", &self.general.font_size.to_string())
            .replace("{display_time}", &self.general.display_time.to_string())
            .replace("{volume}", &self.general.volume.to_string())
            .replace("{hotkey_mod}", &self.general.hotkey_mod)
            .replace("{hotkey_key}", &self.general.hotkey_key);

        // Note: schedule entries use template defaults; user's custom schedule entries
        // are not preserved by this simple template approach. For full preservation,
        // the user should edit config.toml directly.
        std::fs::write(&self.config_path, content).map_err(|e| format!("写入配置文件失败: {e}"))?;
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
}

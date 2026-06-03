use serde::{Deserialize, Serialize};

/// The type of reminder sound to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingType {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "end")]
    End,
    #[serde(rename = "special")]
    Special,
}

impl RingType {
    pub fn display_name(self) -> &'static str {
        match self {
            RingType::Start => "Start",
            RingType::End => "End",
            RingType::Special => "Special",
        }
    }
}

/// A single schedule entry: a time (HH:MM) and the ring type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: String,
    pub ring: RingType,
}

/// Top-level config structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval_secs: u32,
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
}

fn default_interval() -> u32 {
    30
}

impl Config {
    /// Load config from `config.toml`.
    ///
    /// - If the file is missing, write a default one.
    /// - If the schedule is empty, fall back to a single 10:02 Special reminder.
    /// - Chinese colons (：) are silently corrected to English colons (:).
    pub fn load_or_create() -> Result<Self, Box<dyn std::error::Error>> {
        let dir = std::env::current_exe()?
            .parent()
            .ok_or("cannot determine executable directory")?
            .to_path_buf();
        let path = dir.join("config.toml");
        if path.exists() {
            let raw = std::fs::read_to_string(path)?;
            let fixed = fix_chinese_colon(&raw);
            let mut cfg: Config =
                toml::from_str(&fixed).map_err(|e| format!("Failed to parse config.toml: {e}"))?;
            // Normalise every time string to HH:MM.
            for entry in &mut cfg.schedule {
                entry.time = normalise_time(&entry.time);
            }
            // Guard against an empty schedule.
            if cfg.schedule.is_empty() {
                cfg.schedule = default_schedule();
            }
            // Sort once by time so lookups can short-circuit.
            cfg.schedule.sort_by_key(|e| {
                parse_hhmm(&e.time)
                    .map(|(h, m)| h * 60 + m)
                    .unwrap_or(u32::MAX)
            });
            Ok(cfg)
        } else {
            let cfg = Config::default();
            let content = toml::to_string_pretty(&cfg)?;
            std::fs::write(path, content)?;
            Ok(cfg)
        }
    }

    /// Find the next schedule entry strictly after the given time.
    /// Wraps around to the earliest entry tomorrow when past the last one.
    pub fn next_reminder(
        &self,
        current_h: u32,
        current_m: u32,
    ) -> Option<(u32, u32, RingType, String)> {
        let current_total = current_h * 60 + current_m;

        let parsed: Vec<(u32, u32, u32, RingType, String)> = self
            .schedule
            .iter()
            .filter_map(|entry| {
                let (h, m) = parse_hhmm(&entry.time)?;
                Some((h * 60 + m, h, m, entry.ring, entry.time.clone()))
            })
            .collect();

        // First entry strictly after current time (schedule is sorted).
        if let Some(&(_, h, m, ring, ref s)) =
            parsed.iter().find(|(total, ..)| *total > current_total)
        {
            return Some((h, m, ring, s.clone()));
        }

        // Wrap around: first entry of the next day.
        parsed
            .first()
            .map(|&(_, h, m, ring, ref s)| (h, m, ring, s.clone()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval_secs: 30,
            schedule: default_schedule(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse "HH:MM" (or "H:MM", "HH:M") → `(hour, minute)`.
/// Leading zeros are optional; both "9:00" and "09:00" return `(9, 0)`.
pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
}

/// Normalise a time string to "HH:MM" format.
fn normalise_time(s: &str) -> String {
    if let Some((h, m)) = parse_hhmm(s) {
        format!("{h:02}:{m:02}")
    } else {
        s.to_string()
    }
}

/// Replace Chinese colons.
fn fix_chinese_colon(raw: &str) -> String {
    raw.replace('：', ":")
}

/// Default schedule written when no config.toml exists.
fn default_schedule() -> Vec<ScheduleEntry> {
    vec![ScheduleEntry {
        time: "10:02".into(),
        ring: RingType::Special,
    }]
}

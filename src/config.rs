use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: String,
    pub ring: RingType,
}

#[derive(Debug, Clone)]
struct ParsedEntry {
    total: u32,
    hour: u32,
    minute: u32,
    ring: RingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval_secs: u32,
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
    #[serde(skip)]
    entries: Vec<ParsedEntry>,
}

fn default_interval() -> u32 {
    30
}

impl Config {
    pub fn load_or_create() -> Result<Self, Box<dyn std::error::Error>> {
        let dir = std::env::current_exe()?
            .parent()
            .ok_or("cannot determine executable directory")?
            .to_path_buf();
        let path = dir.join("config.toml");

        if path.exists() {
            let raw = std::fs::read_to_string(path)?;
            let fixed = raw.replace('\u{FF1A}', ":");
            let mut cfg: Config =
                toml::from_str(&fixed).map_err(|e| format!("Failed to parse config.toml: {e}"))?;

            cfg.schedule.retain_mut(|entry| {
                if let Some((h, m)) = parse_hhmm(&entry.time) {
                    entry.time = format!("{h:02}:{m:02}");
                    true
                } else {
                    crate::audio::debug_log(&format!(
                        "[tip_clock] ignoring invalid time: {}\n",
                        entry.time
                    ));
                    false
                }
            });

            if cfg.schedule.is_empty() {
                cfg.schedule = default_schedule();
            }

            cfg.rebuild_entries();
            Ok(cfg)
        } else {
            let mut cfg = Config::default();
            cfg.rebuild_entries();
            let content = toml::to_string_pretty(&cfg)?;
            std::fs::write(path, content)?;
            Ok(cfg)
        }
    }

    pub fn next_reminder(&self, current_h: u32, current_m: u32) -> Option<(u32, u32, RingType)> {
        let current_total = current_h * 60 + current_m;
        if let Some(e) = self.entries.iter().find(|e| e.total > current_total) {
            return Some((e.hour, e.minute, e.ring));
        }
        self.entries.first().map(|e| (e.hour, e.minute, e.ring))
    }

    fn rebuild_entries(&mut self) {
        self.entries = self
            .schedule
            .iter()
            .filter_map(|entry| {
                let (h, m) = parse_hhmm(&entry.time)?;
                Some(ParsedEntry {
                    total: h * 60 + m,
                    hour: h,
                    minute: m,
                    ring: entry.ring,
                })
            })
            .collect();
        self.entries.sort_by_key(|e| e.total);
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval_secs: 30,
            schedule: default_schedule(),
            entries: Vec::new(),
        }
    }
}

pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
}

fn default_schedule() -> Vec<ScheduleEntry> {
    vec![ScheduleEntry {
        time: "10:02".into(),
        ring: RingType::Special,
    }]
}

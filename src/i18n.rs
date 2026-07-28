use std::sync::OnceLock;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn detect() -> Self {
        let lang_name = Self::get_system_lang_name();
        // Primary language ID: Chinese = "zh"
        if lang_name.len() >= 2 && &lang_name[..2] == "zh" {
            Lang::Zh
        } else {
            Lang::En
        }
    }

    fn get_system_lang_name() -> String {
        // Use GetUserDefaultLocaleName from kernel32
        unsafe {
            let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
            let len = GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                String::from_utf16_lossy(&buf[..(len as usize - 1)])
            } else {
                String::from("en-US")
            }
        }
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
}

/// Translation keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrKey {
    AppName,
    NextReminder,
    NoMoreReminders,
    ShowClock,
    SkipNext,
    Pause,
    Resume,
    Exit,
    EditConfig,
    TextColor,
    BgColor,
}

/// Lazy-initialized language
static LANG: OnceLock<Lang> = OnceLock::new();

pub fn init() {
    LANG.set(Lang::detect()).ok();
}

pub fn lang() -> Lang {
    *LANG.get().unwrap_or(&Lang::En)
}

pub fn tr(key: TrKey) -> &'static str {
    let lang = lang();
    match key {
        TrKey::AppName => match lang {
            Lang::En => "Tip Clock",
            Lang::Zh => "提示时钟",
        },
        TrKey::NextReminder => match lang {
            Lang::En => "Next",
            Lang::Zh => "下次",
        },
        TrKey::NoMoreReminders => match lang {
            Lang::En => "No more reminders today",
            Lang::Zh => "今日无更多提醒",
        },
        TrKey::ShowClock => match lang {
            Lang::En => "Show Clock",
            Lang::Zh => "显示时钟",
        },
        TrKey::SkipNext => match lang {
            Lang::En => "Skip next",
            Lang::Zh => "跳过下次",
        },
        TrKey::Pause => match lang {
            Lang::En => "Pause",
            Lang::Zh => "暂停",
        },
        TrKey::Resume => match lang {
            Lang::En => "Resume",
            Lang::Zh => "继续",
        },
        TrKey::Exit => match lang {
            Lang::En => "Exit",
            Lang::Zh => "退出",
        },
        TrKey::EditConfig => match lang {
            Lang::En => "Edit Config",
            Lang::Zh => "编辑配置",
        },
        TrKey::TextColor => match lang {
            Lang::En => "Text Color...",
            Lang::Zh => "文字颜色...",
        },
        TrKey::BgColor => match lang {
            Lang::En => "Background Color...",
            Lang::Zh => "背景颜色...",
        },
    }
}

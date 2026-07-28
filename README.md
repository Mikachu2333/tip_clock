# Tip Clock

A lightweight, always-on-top clock overlay for Windows with scheduled reminders and configurable appearance.

轻量级 Windows 时钟悬浮窗，支持定时提醒和自定义外观。

## Features / 功能

- Transparent clock overlay with configurable background color, text color, and opacity
- Scheduled audio reminders with custom WAV support
- Global hotkey to show/hide the clock (default: `Win+Alt+B`)
- System tray with quick-access menu
- Single-instance (no duplicate processes)
- DPI-aware rendering (GDI+)

---

- 可配置背景颜色、文字颜色和透明度的时钟悬浮窗
- 定时音频提醒，支持自定义 WAV 文件
- 全局快捷键显示/隐藏时钟（默认：`Win+Alt+B`）
- 系统托盘图标和右键菜单
- 单实例运行
- DPI 感知渲染（GDI+）

## Installation / 安装

Download `tip_clock.exe` from the [Releases](https://github.com/your/repo/releases) page and place it in any folder. Run it — a `config.toml` will be created on first launch.

从 [Releases](https://github.com/your/repo/releases) 下载 `tip_clock.exe`，放置到任意文件夹后运行。首次启动会自动创建 `config.toml`。

### Build from source / 从源码编译

```bash
# Requires Rust 1.85+ (edition 2024)
git clone https://github.com/your/repo.git
cd tip_clock
cargo build --release
```

## Configuration / 配置

Edit `config.toml` (in the same folder as the EXE) and restart the program.

编辑 EXE 同目录下的 `config.toml` 后重启程序。

```toml
[general]
auto_start = false       # launch on Windows startup / 开机自启

bg_r = 0                 # background RGB (0-255)
bg_g = 0
bg_b = 0
bg_opacity = 100         # background opacity (0-100)

text_r = 255             # text RGB (0-255)
text_g = 255
text_b = 255

display_time = 5         # auto-hide after N seconds (1-60)
volume = 80              # default volume (0-100)

hotkey_mod = "Win+Alt"   # modifiers: alt, ctrl, shift, win
hotkey_key = "B"         # key: A-Z, 0-9, F1-F12, Space, etc.

[[schedule]]
time = "08:00:00"
ring = "start"           # start / end / special / custom / none

[[schedule]]
time = "08:45:00"
ring = "end"

[[schedule]]
time = "09:40:00"
ring = "special"
```

### Ring types / 提示音类型

| Type | Behavior |
|------|----------|
| `start` | Embedded start.wav |
| `end` | Embedded end.wav |
| `special` | Embedded special.wav |
| `custom` | Plays `<custom_file>.wav` from EXE folder |
| `none` | No sound |

## Tray menu / 托盘菜单

- **Show Clock** — toggle the clock overlay
- **Skip next** — skip the upcoming reminder
- **Pause / Resume** — pause or resume all reminders
- **Edit Config** — open config.toml in Notepad
- **Text Color...** — change the clock text color
- **Background Color...** — change the clock background color
- **Exit** — quit the program

---

- **显示时钟** — 切换时钟悬浮窗
- **跳过下次** — 跳过下一次提醒
- **暂停 / 继续** — 暂停或恢复所有提醒
- **编辑配置** — 用记事本打开 config.toml
- **文字颜色...** — 更改时钟文字颜色
- **背景颜色...** — 更改时钟背景颜色
- **退出** — 退出程序

## License / 许可

MIT

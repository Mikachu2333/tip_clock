# Auto Tip Clock

一个自动贴边隐藏、定时弹出提醒的 Windows 桌面时钟。支持自定义提示音、全局快捷键、拖拽定位，配置简单，开箱即用。

A lightweight Windows desktop clock that auto-hides at the screen edge and pops up at scheduled times for reminders. Supports custom audio, global hotkeys, drag-to-reposition, and works out of the box.

---

## 功能 / Features

| 功能          | 说明                                                         |
| ------------- | ------------------------------------------------------------ |
| 定时提醒      | 在 `config.toml` 中设置任意数量的提醒时间，支持 24 小时制    |
| 提示音        | 内置 start / end / special 三种提示音，也支持自定义 WAV 文件 |
| 全局快捷键    | 默认 `Win+Alt+B` 显示/隐藏时钟，可在配置中自定义             |
| 拖拽定位      | 弹出后可直接拖动窗口到任意位置，位置自动保存                 |
| 自动隐藏      | 弹出后按设定时间自动隐藏（默认 5 秒）                        |
| 贴边滑入/滑出 | 带平滑动画的滑入/滑出效果                                    |
| 托盘菜单      | 系统托盘图标，左键切换显示，右键打开菜单                     |
| 颜色自定义    | 支持通过颜色选择器更改文字和背景颜色                         |
| 开机自启      | 可配置 Windows 启动时自动运行                                |
| DPI 感知      | 支持多显示器、高 DPI 环境                                    |
| 单实例        | 防止重复启动                                                 |
| 多语言        | 自动检测系统语言（中文/英文）                                |

---

## 注意事项 / Notes

1. 本软件为免费软件，遵循 MIT 协议。
2. 为降低系统负载，程序每 0.5 秒刷新一次时钟显示。
3. 提醒时间格式为 `HH:MM:SS`（24 小时制），个位数需补零，例如 `08:00:00`。
4. 修改 `config.toml` 后需重启程序生效。
5. 自定义 WAV 文件放在 EXE 同目录下，配置中只需写文件名（不含 `.wav` 后缀）。

---

## 安装 / Installation

从 [Releases](https://github.com/Mikachu2333/AutoTipClock/releases) 下载 `tip_clock.exe`，放到任意文件夹后运行。首次启动会自动创建 `config.toml`。

Download `tip_clock.exe` from the [Releases](https://github.com/Mikachu2333/AutoTipClock/releases) page, place it in any folder, and run it. A `config.toml` will be created on first launch.

### 从源码编译 / Build from source

```bash
# 需要 Rust 1.85+ (edition 2024)
# Requires Rust 1.85+ (edition 2024)
git clone https://github.com/Mikachu2333/AutoTipClock.git
cd AutoTipClock
cargo build --release
```

---

## 配置说明 / Configuration

编辑 EXE 同目录下的 `config.toml`，保存后重启程序生效。

Edit `config.toml` (in the same folder as the EXE) and restart the program.

```toml
[general]
# 开机自启 / Launch on Windows startup
auto_start = false

# 背景颜色 RGB (0-255)，透明度 (0-100, 0=全透明)
# Background color RGB (0-255), opacity (0-100, 0=fully transparent)
bg_r = 0
bg_g = 0
bg_b = 0
bg_opacity = 0

# 文字颜色 RGB (0-255)
# Text color RGB (0-255)
text_r = 0
text_g = 0
text_b = 0

# 弹出后自动隐藏的秒数 (1-60)
# Auto-hide after N seconds (1-60)
display_time = 5

# 默认音量 (0-100)
# Default volume (0-100)
volume = 80

# 显示/隐藏快捷键
# 修饰键: alt, ctrl, shift, win（可用 + 组合）
# Show/hide hotkey
# Modifiers: alt, ctrl, shift, win (use + to combine)
hotkey_mod = "Win+Alt"
hotkey_key = "B"

# 窗口位置（左上角像素坐标，-1 表示自动定位）
# Window position (top-left pixel coordinates, -1 = auto)
window_x = -1
window_y = -1

# ── 提醒时间 / Reminder schedule ──────────────
# 每个 [[schedule]] 块定义一个提醒
# Each [[schedule]] block defines one reminder
#
# time: HH:MM:SS (24小时制)
# ring: start / end / special / custom / none
#       start   = 内置开始提示音 / built-in start chime
#       end     = 内置结束提示音 / built-in end chime
#       special = 内置特别提示音 / built-in special chime
#       custom  = 播放自定义 WAV / play custom WAV (需设置 custom_file)
#       none    = 静音 / silent (仅弹窗)

[[schedule]]
time = "08:00:00"
ring = "start"

[[schedule]]
time = "08:45:00"
ring = "end"

[[schedule]]
time = "09:40:00"
ring = "special"

# 自定义提示音示例 / Custom audio example
# [[schedule]]
# time = "12:00:00"
# ring = "custom"
# custom_file = "lunch"   # 播放 EXE 目录下的 lunch.wav
```

### 提示音类型 / Ring Types

| 类型      | 说明                                                     |
| --------- | -------------------------------------------------------- |
| `start`   | 内置开始提示音                                           |
| `end`     | 内置结束提示音                                           |
| `special` | 内置特别提示音                                           |
| `custom`  | 播放 EXE 目录下的自定义 WAV 文件（需设置 `custom_file`） |
| `none`    | 静音，仅弹出窗口                                         |

---

## 托盘菜单 / Tray Menu

| 菜单项                   | 功能                     |
| ------------------------ | ------------------------ |
| 下次 / Next              | 显示下一次提醒时间和类型 |
| 显示时钟 / Show Clock    | 切换时钟显示/隐藏        |
| 跳过下次 / Skip Next     | 跳过下一次提醒           |
| 暂停 / Resume            | 暂停/恢复所有提醒        |
| 编辑配置 / Edit Config   | 用记事本打开 config.toml |
| 文字颜色... / Text Color | 更改时钟文字颜色         |
| 背景颜色... / Bg Color   | 更改时钟背景颜色         |
| 退出 / Exit              | 退出程序                 |

**左键点击托盘图标** = 切换时钟显示/隐藏
**右键点击托盘图标** = 打开菜单

---

## 快捷键 / Hotkeys

| 快捷键              | 功能          |
| ------------------- | ------------- |
| `Win+Alt+B`（默认） | 显示/隐藏时钟 |

可在 `config.toml` 中自定义，支持的修饰键：`alt`、`ctrl`、`shift`、`win`，支持的按键：`A-Z`、`0-9`、`F1-F12`、`Space`、`Enter`、`Esc` 等。

---

## 项目结构 / Project Structure

```tree
src/
├── main.rs      — 程序入口、托盘菜单、消息循环
├── config.rs    — TOML 配置解析、时间规范化
├── audio.rs     — 音频播放（PlaySoundW）
├── gui.rs       — 时钟窗口（GDI+ 渲染、DPI 感知）
├── hotkey.rs    — 全局快捷键（WH_KEYBOARD_LL）
└── i18n.rs      — 多语言支持（中文/英文）
res/
├── start.wav    — 内置提示音
├── end.wav
└── special.wav
└── ico_raw      — 托盘图标（256×256 RGBA）
```

---

## 许可证 / License

[MIT](LICENSE)

**Author:** Mikachu2333

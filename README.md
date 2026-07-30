# Auto Tip Clock

**[English](./README_EN.md)**

一个自动贴边隐藏、定时弹出提醒的 Windows 桌面时钟。支持自定义提示音、全局快捷键、拖拽定位，配置简单，开箱即用。

---

## 功能

| 功能          | 说明                                                                 |
| ------------- | -------------------------------------------------------------------- |
| 定时提醒      | 在 `config.toml` 中设置任意数量的提醒时间，支持 24 小时制            |
| 提示音        | 每个提醒可选用配置目录中的 WAV、FLAC 或 MP3 文件，也可设置为静音提醒 |
| 全局快捷键    | 默认 `Ctrl+Alt+B` 显示/隐藏时钟，可在配置中自定义                    |
| 拖拽定位      | 弹出后可直接拖动窗口到任意位置，位置自动保存                         |
| 自动隐藏      | 弹出后按设定时间自动隐藏（默认 3 秒）                                |
| 贴边滑入/滑出 | 带平滑动画的滑入/滑出效果                                            |
| 托盘菜单      | 系统托盘图标，左键切换显示，右键打开菜单                             |
| 颜色自定义    | 支持通过颜色选择器更改文字和背景颜色                                 |
| 开机自启      | 可配置 Windows 启动时自动运行                                        |
| DPI 感知      | 支持多显示器、高 DPI 环境                                            |
| 单实例        | 防止重复启动                                                         |
| 多语言        | 自动检测系统语言（中文/英文）                                        |

---

## 注意事项

1. 本软件为免费软件，遵循 MIT 协议。
2. 为降低系统负载，程序每 0.5 秒刷新一次时钟显示。
3. 提醒时间格式为 `HH:MM:SS`（24 小时制），个位数需补零，例如 `08:00:00`。
4. `display_time` 和 `[[schedule]]` 支持运行时热重载并自动去重；`hotkey_*`、`auto_start`、`volume` 修改后需重启。颜色、透明度和窗口位置请通过程序界面修改。
5. WAV、FLAC、MP3 文件放在 `config.toml` 所在目录，三种格式都可省略扩展名。名称相同时按 WAV → FLAC → MP3 的顺序选择；写明扩展名则使用指定文件。不允许绝对路径或子目录。
6. 程序优先在 EXE 目录保存配置；若该目录不可写，则自动使用 `%LOCALAPPDATA%\TipClock\config.toml`。首次创建配置时，会在同一目录释放示例音频 `demo.mp3`。
7. `volume` 仅控制 Tip Clock 自身的音频输出，不会修改 Windows 系统音量。

---

## 安装

从 [Releases](https://github.com/Mikachu2333/tip_clock/releases) 下载 `tip_clock.exe`，放到任意文件夹后运行。首次启动会自动创建 `config.toml` 和示例音频 `demo.mp3`。

### 从源码编译

```bash
# 需要 Rust 1.85+ (edition 2024)
git clone https://github.com/Mikachu2333/tip_clock.git --depth=1
cd tip_clock
cargo build --release
```

## 配置说明

编辑实际配置目录中的 `config.toml`，保存后重启程序生效。EXE 目录不可写时，实际配置目录为 `%LOCALAPPDATA%\TipClock`。

```toml
[general]
# 开机自启
auto_start = false

# 背景颜色 RGB (0-255)，不透明度 (0-100, 0=全透明)
bg_r = 255
bg_g = 255
bg_b = 255
bg_opacity = 0

# 文字颜色 RGB (0-255)
text_r = 0
text_g = 0
text_b = 0

# 弹出后自动隐藏的秒数 (1-60)
display_time = 3

# 默认音量 (0-100)
volume = 80

# 显示/隐藏快捷键
# 修饰键: alt, ctrl, shift, win（可用 + 组合）
hotkey_mod = "Ctrl+Alt"
hotkey_key = "B"

# 窗口位置（左上角像素坐标，-1 表示自动定位）
window_x = -1
window_y = -1

# ── 提醒时间 ──────────────────────────────────
# 每个 [[schedule]] 块定义一个提醒
#
# time: HH:MM:SS (24小时制)
# audio: 可选，配置目录内的 WAV / FLAC / MP3 文件名

# 有声提醒；省略扩展名时依次查找 demo.wav、demo.flac、demo.mp3
[[schedule]]
time = "08:00:00"
audio = "demo"

# 静音提醒：省略 audio，仅弹出时钟
[[schedule]]
time = "13:42:57"
```

### 提示音配置

- `audio` 是可选字段；省略时为静音提醒。
- 支持 WAV、FLAC、MP3。
- 音频文件位于 `config.toml` 所在目录。
- 首次生成配置时会同时释放 `demo.mp3` 作为示例。

---

## 托盘菜单

| 菜单项              | 功能                           |
| ------------------- | ------------------------------ |
| 下次                | 显示下一次提醒时间和类型       |
| 显示时钟 / 隐藏时钟 | 切换时钟显示/隐藏              |
| 跳过下次            | 跳过下一次提醒                 |
| 暂停 / 继续         | 暂停/恢复所有提醒              |
| 编辑配置            | 用记事本打开 config.toml       |
| 文字颜色...         | 更改时钟文字颜色               |
| 背景颜色...         | 更改时钟背景颜色               |
| 不透明度            | 通过数值输入框调整背景不透明度 |
| 退出                | 退出程序                       |

**左键点击托盘图标** = 切换时钟显示/隐藏
**右键点击托盘图标** = 打开菜单

---

## 快捷键

| 快捷键               | 功能          |
| -------------------- | ------------- |
| `Ctrl+Alt+B`（默认） | 显示/隐藏时钟 |

可在 `config.toml` 中自定义，支持的修饰键：`alt`、`ctrl`、`shift`、`win`，支持的按键：`A-Z`、`0-9`、`F1-F12`、`Space`、`Enter`、`Esc` 等。

---

## 项目结构

```tree
src/
├── main.rs      — 程序入口、托盘菜单、消息循环
├── config.rs    — TOML 配置解析、时间规范化
├── audio.rs     — 外部 WAV / FLAC / MP3 播放（rodio）
├── gui.rs       — 时钟窗口（GDI+ 渲染、DPI 感知）
├── hotkey.rs    — 全局快捷键（RegisterHotKey）
└── i18n.rs      — 多语言支持（中文/英文）
res/
├── demo.mp3     — 首次创建配置时释放的示例音频
└── ico_raw      — 托盘图标（256×256 RGBA）
```

---

## 许可证

[MIT](./LICENSE)

**Author:** Mikachu2333

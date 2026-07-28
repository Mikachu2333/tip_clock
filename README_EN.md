# Auto Tip Clock

**[中文](./README.md)**

A lightweight Windows desktop clock that auto-hides at the screen edge and pops up at scheduled times for reminders. Supports custom audio, global hotkeys, drag-to-reposition, and works out of the box.

---

## Features

| Feature             | Description                                                              |
| ------------------- | ------------------------------------------------------------------------ |
| Scheduled Reminders | Set any number of reminder times in `config.toml` (24-hour format)       |
| Audio Alerts        | Optional WAV, FLAC, or MP3 file per reminder; silent reminders supported |
| Global Hotkey       | Show/hide clock with `Win+Alt+B` (customizable)                          |
| Drag to Reposition  | Drag the popup window anywhere; position is saved automatically          |
| Auto Hide           | Hides automatically after a configurable duration (default 3s)           |
| Slide Animation     | Smooth slide-in / slide-out animation                                    |
| Tray Menu           | System tray icon - left-click to toggle, right-click for menu            |
| Color Customization | Change text and background colors via color picker                       |
| Auto Start          | Optional Windows startup launch                                          |
| DPI Aware           | Supports multi-monitor and high-DPI setups                               |
| Single Instance     | Prevents duplicate processes                                             |
| Multi-language      | Auto-detects system language (Chinese / English)                         |

---

## Notes

1. This is free software released under the MIT license.
2. The clock display refreshes every 0.5 seconds to minimize system load.
3. Time format is `HH:MM:SS` (24-hour), e.g. `08:00:00`. Single digits must be zero-padded.
4. Changes to `config.toml` require a program restart to take effect.
5. WAV, FLAC, and MP3 files must be in the `config.toml` directory, and all three may omit the extension. For identical stems, lookup order is WAV → FLAC → MP3; an explicit extension selects that file. Absolute paths and subdirectories are rejected.
6. The EXE directory is preferred for configuration. If it is not writable, `%LOCALAPPDATA%\TipClock\config.toml` is used automatically. When configuration is first created, `demo.mp3` is extracted into the same directory.
7. `volume` controls only Tip Clock's audio stream and never changes the Windows system volume.

---

## Installation

Download `tip_clock.exe` from the [Releases](https://github.com/Mikachu2333/tip_clock/releases) page, place it in any folder, and run it. `config.toml` and the example audio `demo.mp3` are created on first launch.

### Build from source

```bash
# Requires Rust 1.85+ (edition 2024)
git clone https://github.com/Mikachu2333/tip_clock.git --depth=1
cd tip_clock
cargo build --release
```

---

## Configuration

Edit `config.toml` in the active configuration directory and restart the program. If the EXE directory is not writable, the active directory is `%LOCALAPPDATA%\TipClock`.

```toml
[general]
# Launch on Windows startup
auto_start = false

# Background color RGB (0-255), opacity (0-100, 0=fully transparent, 100=solid)
bg_r = 255
bg_g = 255
bg_b = 255
bg_opacity = 0

# Text color RGB (0-255)
text_r = 0
text_g = 0
text_b = 0

# Auto-hide after N seconds (1-60)
display_time = 3

# Default volume (0-100)
volume = 80

# Show/hide hotkey
# Modifiers: alt, ctrl, shift, win (use + to combine)
hotkey_mod = "Win+Alt"
hotkey_key = "B"

# Window position (top-left pixel coordinates, -1 = auto)
window_x = -1
window_y = -1

# ── Reminder schedule ─────────────────────────
# Each [[schedule]] block defines one reminder.
#
# time: HH:MM:SS (24-hour)
# audio: optional WAV / FLAC / MP3 file in the config directory

# Audible reminder; omitted extension searches demo.wav, demo.flac, demo.mp3
[[schedule]]
time = "08:00:00"
audio = "demo"

# Silent reminder: omit audio
[[schedule]]
time = "12:00:00"
```

### Audio Configuration

- `audio` is optional; omitting it creates a silent visual reminder.
- WAV, FLAC, and MP3 are supported.
- Audio files reside beside `config.toml`.
- `demo.mp3` is extracted when the initial configuration is created.

---

## Tray Menu

| Item                    | Action                                       |
| ----------------------- | -------------------------------------------- |
| Next                    | Show next reminder time and type             |
| Show Clock / Hide Clock | Toggle clock visibility                      |
| Skip Next               | Skip the upcoming reminder                   |
| Pause / Resume          | Pause or resume all reminders                |
| Edit Config             | Open config.toml in Notepad                  |
| Text Color...           | Change clock text color                      |
| Background Color...     | Change clock background color                |
| Opacity                 | Adjust background opacity in a numeric field |
| Exit                    | Quit the program                             |

**Left-click** the tray icon = toggle clock
**Right-click** the tray icon = open menu

---

## Hotkeys

| Hotkey                | Action            |
| --------------------- | ----------------- |
| `Win+Alt+B` (default) | Show / hide clock |

Customizable in `config.toml`. Supported modifiers: `alt`, `ctrl`, `shift`, `win`. Supported keys: `A-Z`, `0-9`, `F1-F12`, `Space`, `Enter`, `Esc`, etc.

---

## Project Structure

```tree
src/
├── main.rs      — Entry point, tray menu, message loop
├── config.rs    — TOML config parsing, time normalization
├── audio.rs     — External WAV / FLAC / MP3 playback (rodio)
├── gui.rs       — Clock window (GDI+ rendering, DPI aware)
├── hotkey.rs    — Global hotkey (RegisterHotKey)
└── i18n.rs      — Multi-language support (Chinese / English)
res/
├── demo.mp3     — Example audio extracted with the initial config
└── ico_raw      — Tray icon (256×256 RGBA)
```

---

## License

[MIT](./LICENSE)

**Author:** Mikachu2333

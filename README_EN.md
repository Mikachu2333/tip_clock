# Auto Tip Clock

**[中文](./README.md)**

A lightweight Windows desktop clock that auto-hides at the screen edge and pops up at scheduled times for reminders. Supports custom audio, global hotkeys, drag-to-reposition, and works out of the box.

---

![Example](./PixPin_2026-07-31_09-22-28.mp4)

## Features

| Feature             | Description                                                              |
| ------------------- | ------------------------------------------------------------------------ |
| Scheduled Reminders | Set any number of reminder times in `config.toml` (24-hour format)       |
| Audio Alerts        | Optional WAV, FLAC, or MP3 file per reminder; silent reminders supported |
| Global Hotkey       | Show/hide clock with `Ctrl+Alt+B` (customizable)                         |
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
2. Time format is `HH:MM:SS` (24-hour), e.g. `08:00:00`. Single digits must be zero-padded.
3. `display_time` and `[[schedule]]` are hot-reloaded and deduplicated at runtime. Changes to `hotkey_*`, `auto_start`, and `volume` require a restart. Change colors, opacity, and window position through the application.
4. WAV, FLAC, and MP3 files must be in the `config.toml` directory, and all three may omit the extension. For identical stems, lookup order is WAV → FLAC → MP3; an explicit extension selects that file. Absolute paths and subdirectories are rejected.
5. The EXE directory is preferred for configuration. If it is not writable, `%LOCALAPPDATA%\TipClock\config.toml` is used automatically. When configuration is first created, `demo.mp3` is extracted into the same directory.
6. `volume` controls only Tip Clock's audio stream and never changes the Windows system volume.

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

Edit `config.toml` in the active configuration directory. `display_time` and `[[schedule]]` hot-reload within about one second; changes to `hotkey_*`, `auto_start`, and `volume` require a restart. Change colors, background opacity, and window position through the application; those values are written back immediately. If the EXE directory is not writable, the active directory is `%LOCALAPPDATA%\TipClock`.

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
hotkey_mod = "Ctrl+Alt"
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
time = "13:42:57"
```

### Audio Configuration

- `audio` is optional; omitting it creates a silent visual reminder.
- WAV, FLAC, and MP3 are supported.
- Audio files reside beside `config.toml`.
- `demo.mp3` is extracted when the initial configuration is created.

### Multiple Reminders at the Same Time

Entries with the same `time` form one reminder group. If one timestamp contains a silent entry plus `audio = "A"` and `audio = "B"`:

- The clock appears once; the silent entry does not create another popup.
- A and B are queued in configuration order, not played simultaneously or interrupted.
- Identical `time + audio` entries are deduplicated; different audio at the same time is retained.
- “Skip next reminder group” skips the whole timestamp group. A group missed while paused is not replayed.

---

## Tray Menu

| Item                     | Action                                       |
| ------------------------ | -------------------------------------------- |
| Next                     | Show the next group time and audible marker  |
| Show Clock / Hide Clock  | Toggle clock visibility                      |
| Skip next reminder group | Skip every reminder at the next timestamp    |
| Pause / Resume           | Pause or resume all reminders                |
| Edit Config              | Open config.toml in Notepad                  |
| Text Color...            | Change clock text color                      |
| Background Color...      | Change clock background color                |
| Opacity                  | Adjust background opacity in a numeric field |
| Restart                  | Restart Tip Clock                            |
| Exit                     | Quit the program                             |

**Left-click** the tray icon = toggle clock
**Right-click** the tray icon = open menu

---

## Hotkeys

| Hotkey                 | Action            |
| ---------------------- | ----------------- |
| `Ctrl+Alt+B` (default) | Show / hide clock |

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
├── icon_raw     — Tray and application icon (256×256 raw RGBA)
└── icon.afdesign — Editable icon source
```

---

## License

[MIT](./LICENSE)

**Author:** Mikachu2333

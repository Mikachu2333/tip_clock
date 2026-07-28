# Tip Clock

A Windows system-tray tool that displays a transparent floating clock and plays
reminder sounds on a configurable schedule.

## Features

- **Transparent clock overlay** — semi-transparent background, fully opaque text,
  auto-hides after a configurable timeout.
- **Tray icon** — left-click toggles the clock, right-click opens menu.
- **Global hotkey** — `SetWindowsHookEx` based, supports Win-key combinations.
  Default `Ctrl+Alt+T`, configurable.
- **Font & color** — choose via system dialogs from the tray menu (Font… / Text Color…).
  Settings persisted to `config.toml`.
- **Scheduled reminders** — play WAV sounds at specific times.
  Built-in: start / end / special. Custom WAV files in the EXE directory.
- **i18n** — auto-detects system language (English / 中文).
- **Auto-start** — optional registry entry.
- **Single instance** — prevents multiple copies from running.

## Quick start

1. Place `tip_clock.exe` anywhere and run it.
2. A `config.toml` is created automatically.
3. Edit `config.toml` to customize times, sounds, colors, hotkey.
4. **Restart the program** to apply changes (no hot-reload).

## Building from source

```bash
# Rust 1.85+ (edition 2024)
cargo build --release
# Binary: target/release/tip_clock.exe  (~5.5 MB)
```

## Configuration

```toml
[general]
auto_start = false
bg_opacity = 80          # background opacity 0–100
display_time = 5          # auto-hide after N seconds
volume = 80               # 0–100
hotkey_mod = "ctrl+alt"   # empty = single key (F1–F12 only)
hotkey_key = "T"
font_name = "微软雅黑"     # managed via tray menu
font_size = 16

[[schedule]]
time = "08:00:00"
ring = "start"            # start | end | special | custom | none
```

Time values auto-correct: `9:00` → `09:00:00`, `930` → `09:30:00`.

### Hotkey modifier format

| Value | Meaning |
|-------|---------|
| `""` (empty) | Single key (F1–F12 only) |
| `"ctrl+alt"` | Ctrl + Alt |
| `"win+shift"` | Win + Shift |
| `"ctrl+shift+alt"` | Three modifiers |

## Debugging

Debug build (`cargo build`) enables console output with tagged logs:

```
[main] entering main loop
[hotkey] keyboard hook matched, posting WM_USER_HOTKEY
[gui] WM_HOTKEY / WM_USER_HOTKEY received
[main] schedule match at 09:40:00, ring=Special
[main] tray menu clicked: MenuId("show_clock")
```

## License

MIT

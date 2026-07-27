# Tip Clock

A Windows system-tray tool that displays a transparent floating clock and plays
reminder sounds on a configurable schedule.

## Features

- **Transparent clock overlay** — semi-transparent background, fully opaque text,
  drifts in/out with slide animation, auto-hides after a configurable timeout.
- **Tray icon** — left-click toggles the clock, right-click opens menu (Skip, Pause,
  Edit Config, Exit).
- **Global hotkey** — default `Ctrl+Alt+T`, configurable.
- **Scheduled reminders** — play WAV sounds at specific times of day.
  Built-in: start / end / special. Supports custom WAV files placed next to the EXE.
- **Config hot-reload** — edit `config.toml`, changes apply within seconds.
- **i18n** — auto-detects system language (English / 中文), localizes tray menu
  and config file template.
- **Auto-start** — optional registry entry.
- **Single instance** — prevents multiple copies from running.

## Quick start

1. Download `tip_clock.exe` and place it anywhere.
2. Run it. A `config.toml` is created automatically with default settings.
3. Edit `config.toml` to customize times, sounds, colors, hotkey, etc.
4. Changes take effect automatically — no restart needed.

## Building from source

```bash
# Requires Rust 1.85+ (edition 2024)
cargo build --release
# Binary: target/release/tip_clock.exe  (~5.5 MB)
```

## Configuration

See `config.toml` (auto-generated on first run). Key settings:

```toml
[general]
auto_start = false
bg_opacity = 80        # background opacity (0-100)
display_time = 5        # auto-hide after N seconds
volume = 80             # 0-100
hotkey_mod = "ctrl+alt"
hotkey_key = "T"

[[schedule]]
time = "08:00:00"
ring = "start"          # start | end | special | custom | none
```

Time values are auto-corrected: `9:00` → `09:00:00`, `930` → `09:30:00`.

## License

MIT

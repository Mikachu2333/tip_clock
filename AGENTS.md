# AGENTS.md

## Build & toolchain

- **Edition 2024** — stable Rust 1.85+.
- Windows-only. Uses `user32`, `kernel32`, `shcore`, `advapi32`, `comdlg32`, `gdi32`, `gdiplus` FFI directly.
- Clippy-clean; `upper_case_acronyms` allowed via `#![allow(...)]` (Win32 types match C API convention).

```bash
cargo build --release    # release: LTO, single-cgu, stripped, panic=abort, no console window
cargo build              # debug: console visible, debug_log active
cargo test               # 19 unit tests — config parsing/hot reload, hotkey parsing, audio resolution
cargo clippy             # zero warnings
```

## Project structure

```tree
src/
├── main.rs      — entry point, single-instance, tray icon + menu, message pump,
│                  auto-start registry, ChooseColor dialogs
├── config.rs    — TOML config (auto-create, CJK punctuation auto-correct,
│                  time normalizer, i18n config templates, value clamping)
├── audio.rs     — rodio playback for external WAV/FLAC/MP3 files
├── gui.rs       — layered clock window (WS_EX_LAYERED, GDI+ rendering,
│                  32-bit BGRA DIB, pre-multiplied alpha, DPI-aware text sizing)
├── hotkey.rs    — global hotkey via RegisterHotKey + MOD_NOREPEAT
└── i18n.rs      — GetUserDefaultLocaleName detection (EN/ZH), tray menu + config template i18n
res/
├── demo.mp3     — embedded only for first-run extraction beside config.toml
├── icon_raw     — 256×256 raw RGBA icon used by the tray and Windows resources
└── icon.afdesign — editable icon source
```

## Configuration (`config.toml`)

- Uses the EXE directory when writable, otherwise `%LOCALAPPDATA%\TipClock`. Limited hot reload: `display_time` and deduplicated `[[schedule]]`; hotkey/auto-start/volume require restart; colors/opacity/window position are runtime-owned.
- Auto-created on first run with a language-aware template (ZH or EN); `demo.mp3` is extracted beside it.
- Auto-corrects CJK full-width punctuation and bare time values.
- Missing keys filled from `GeneralConfig::default()`.
- Values clamped: `bg_opacity` <= 100, `display_time` 1-60, `volume` <= 100.

| Section        | Key                            | Type                                                | Default         |
| -------------- | ------------------------------ | --------------------------------------------------- | --------------- |
| `[general]`    | `auto_start`                   | bool                                                | `false`         |
|                | `bg_r` / `bg_g` / `bg_b`       | u8 (0-255)                                          | `255, 255, 255` |
|                | `bg_opacity`                   | u8 (0-100)                                          | `0`             |
|                | `text_r` / `text_g` / `text_b` | u8 (0-255)                                          | `0, 0, 0`       |
|                | `display_time`                 | u32 (1-60 s)                                        | `3`             |
|                | `volume`                       | u8 (0-100)                                          | `80`            |
|                | `hotkey_mod`                   | string                                              | `"Ctrl+Alt"`    |
|                | `hotkey_key`                   | string                                              | `"B"`           |
|                | `window_x`                     | i32                                                 | `-1` (auto)     |
|                | `window_y`                     | i32                                                 | `-1` (auto)     |
| `[[schedule]]` | `time`                         | `"HH:MM:SS"`                                        | -               |
|                | `audio`                        | WAV/FLAC/MP3 file name (optional; omitted = silent) | -               |

## Architecture

- **Single-instance**: via `single_instance` crate (GUID).
- **GUI rendering**: GDI+ (`gdiplus.dll`) renders directly to a 32-bit BGRA DIB with pre-multiplied alpha. `UpdateLayeredWindow` + `ULW_ALPHA` displays the result. No post-processing hacks; GDI+ handles alpha natively.
- **Font**: hardcoded Microsoft YaHei UI at 24pt. The displayed format is `HH : MM : SS`; GDI+ centers the text and a fixed 2px downward offset corrects its visual position.
- **Window sizing**: `GdipMeasureString` measures `88 : 88 : 88` at startup; only the measurement overhang pads (4px width, 2px height) are retained.
- **DPI**: `PROCESS_PER_MONITOR_DPI_AWARE`. Font size and window dimensions scale with system DPI.
- **Message loop**: `MsgWaitForMultipleObjects` (500ms timeout) + `PeekMessageW` / `DispatchMessageW`.
- **Timer**: `WM_TIMER` every 500ms redraws time and checks auto-hide.
- **Hotkey**: OS-managed `RegisterHotKey` with `MOD_NOREPEAT`.
- **Audio**: rodio application-scoped output; external WAV/FLAC/MP3 files beside the active config. No built-in reminder sounds.
- **Auto-start**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` entry.
- **i18n**: `GetUserDefaultLocaleName` detection (EN/ZH). Tray menu and config template follow system language.
- **Tray**: left-click toggles clock; right-click shows context menu. `ChooseColorW` dialogs update text/background colors immediately and persist them to `config.toml`; the opacity panel persists on close.
- **Reminder groups**: all entries at one timestamp form a group; the clock shows once and audio files queue in config order. Duplicate `(time, audio)` entries are removed.
- **Skip next group**: `SKIP_COUNT` counts timestamp groups, not entries. The scheduler consumes one count for the next matched group; menu preview groups by `total_sec` using identical semantics.
- **Menu refresh**: `NEED_REFRESH` is checked every main-loop iteration (≤500ms), providing immediate preview updates after skipping a group.
- **Debug logging**: `cfg!(debug_assertions)` only; `[main]` / `[gui]` / `[hotkey]` prefixes.

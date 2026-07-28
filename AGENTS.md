# AGENTS.md

## Build & toolchain

- **Edition 2024** — stable Rust 1.85+.
- Windows-only. Uses `user32`, `kernel32`, `winmm`, `shcore`, `advapi32`, `comdlg32`, `gdiplus` FFI directly.
- Clippy-clean; `upper_case_acronyms` allowed via `#![allow(...)]` (Win32 types match C API convention).

```bash
cargo build --release    # release: LTO, single-cgu, stripped, panic=abort, no console window
cargo build              # debug: console visible, debug_log active
cargo test               # 8 unit tests for config time-parsing & normalization
cargo clippy             # zero warnings
```

## Project structure

```
src/
├── main.rs      — entry point, single-instance, tray icon + menu, message pump,
│                  auto-start registry, ChooseColor dialogs
├── config.rs    — TOML config (auto-create, CJK punctuation auto-correct,
│                  time normalizer, i18n config templates, value clamping)
├── audio.rs     — PlaySoundW (embedded + custom WAVs), waveOutSetVolume
├── gui.rs       — layered clock window (WS_EX_LAYERED, GDI+ rendering,
│                  32-bit BGRA DIB, pre-multiplied alpha, DPI-aware text sizing)
├── hotkey.rs    — global hotkey via SetWindowsHookEx (WH_KEYBOARD_LL)
└── i18n.rs      — GetUserDefaultLocaleName detection (EN/ZH), tray menu + config template i18n
res/
├── start.wav    — embedded via include_bytes!
├── end.wav
└── special.wav
config.toml      — generated on first run in EXE directory
```

## Configuration (`config.toml`)

- Read **once at startup** from EXE directory. No hot-reload; restart to apply changes.
- Auto-created on first run with a language-aware template (ZH or EN).
- Auto-corrects CJK full-width punctuation and bare time values.
- Missing keys filled from `GeneralConfig::default()`.
- Values clamped: `bg_opacity` <= 100, `display_time` 1-60, `volume` <= 100.

| Section | Key | Type | Default |
|---------|-----|------|---------|
| `[general]` | `auto_start` | bool | `false` |
| | `bg_r` / `bg_g` / `bg_b` | u8 (0-255) | `0, 0, 0` |
| | `bg_opacity` | u8 (0-100) | `100` |
| | `text_r` / `text_g` / `text_b` | u8 (0-255) | `255, 255, 255` |
| | `display_time` | u32 (1-60 s) | `5` |
| | `volume` | u8 (0-100) | `80` |
| | `hotkey_mod` | string | `"Win+Alt"` |
| | `hotkey_key` | string | `"B"` |
| `[[schedule]]` | `time` | `"HH:MM:SS"` | - |
| | `ring` | `start`/`end`/`special`/`custom`/`none` | - |
| | `custom_file` | string (optional) | - |

## Architecture

- **Single-instance**: via `single_instance` crate (GUID).
- **GUI rendering**: GDI+ (`gdiplus.dll`) renders directly to a 32-bit BGRA DIB with pre-multiplied alpha. `UpdateLayeredWindow` + `ULW_ALPHA` displays the result. No post-processing hacks; GDI+ handles alpha natively.
- **Font**: hardcoded (18pt, DPI-scaled). No longer configurable via config or dialog.
- **Window sizing**: text is rendered to a temporary bitmap at startup; pixel scan measures exact bounds. `GdipGetFontHeight` provides the floor for height.
- **DPI**: `PROCESS_PER_MONITOR_DPI_AWARE`. Font size and window dimensions scale with system DPI.
- **Message loop**: `MsgWaitForMultipleObjects` (500ms timeout) + `PeekMessageW` / `DispatchMessageW`.
- **Timer**: `WM_TIMER` every 500ms redraws time and checks auto-hide.
- **Hotkey**: `SetWindowsHookEx(WH_KEYBOARD_LL)` global hook posts `WM_USER_HOTKEY` to clock window.
- **Audio**: `PlaySoundW` with `SND_MEMORY | SND_ASYNC` (embedded) or `SND_FILENAME | SND_ASYNC` (custom). Volume via `waveOutSetVolume`.
- **Auto-start**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` entry.
- **i18n**: `GetUserDefaultLocaleName` detection (EN/ZH). Tray menu and config template follow system language.
- **Tray**: left-click toggles clock; right-click shows context menu. `ChooseColorW` dialogs for text/background color.
- **Debug logging**: `cfg!(debug_assertions)` only; `[main]` / `[gui]` / `[hotkey]` prefixes.

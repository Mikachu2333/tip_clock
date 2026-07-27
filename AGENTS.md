# AGENTS.md

## Build & toolchain

- **Edition 2024** — uses stable Rust 1.85+ (no longer requires nightly).
- Windows-only. Uses `user32`, `kernel32`, `winmm`, `shcore`, `advapi32` FFI directly.
- Clippy-clean; all `upper_case_acronyms` allowed via module-level `#![allow(...)]` (Win32 types match C API convention).

```bash
cargo build --release    # release: LTO, single-cgu, stripped, panic=abort, no console window
cargo build              # debug: console window visible
cargo test               # 8 unit tests for config time-parsing
cargo clippy             # zero warnings
```

## Project structure

```tree
src/
├── main.rs      — entry point, single-instance, tray icon + menu, message pump, auto-start
├── config.rs    — TOML config (auto-create, hot-reload, CJK punctuation auto-correct, time normalizer)
├── audio.rs     — PlaySoundW playback (embedded + custom WAVs), waveOut volume control
├── gui.rs       — transparent layered clock window (WS_EX_LAYERED, per-pixel alpha, GDI text)
├── hotkey.rs    — RegisterHotKey global shortcut
└── i18n.rs      — system locale detection (EN/ZH), tray menu translation, config template i18n
res/
├── start.wav    — embedded at compile time via include_bytes!
├── end.wav
└── special.wav
config.toml      — generated on first run in EXE directory (language-aware template)
```

## Configuration (`config.toml`)

- Read from **the EXE's directory** (not CWD).
- Auto-created on first run using a language-aware template (ZH or EN based on `GetUserDefaultLocaleName`).
- **Hot-reload**: checked every 5 seconds via file modification time. No restart needed.
- Auto-corrects common mistakes:
  - CJK full-width punctuation (`：` → `:`, `＝` → `=`, etc.)
  - Bare time values without quotes (`time = 09:00` → `time = "09:00"`)
  - Time format normalization (`9` → `09:00:00`, `930` → `09:30:00`, `090000` → `09:00:00`)
- Missing keys filled from defaults (`GeneralConfig::default()`).
- Schedule entries with unparseable times are logged and dropped.

### Config sections

| Section | Key | Type | Default |
| --------- | ----- | ------ | --------- |
| `[general]` | `auto_start` | bool | `false` |
| | `bg_r` / `bg_g` / `bg_b` | u8 (0-255) | `0, 0, 0` |
| | `bg_opacity` | u8 (0-100) | `80` |
| | `text_r` / `text_g` / `text_b` | u8 (0-255) | `255, 255, 255` |
| | `display_time` | u32 (1-60s) | `5` |
| | `volume` | u8 (0-100) | `80` |
| | `hotkey_mod` | string | `"ctrl+alt"` |
| | `hotkey_key` | string | `"T"` |
| `[[schedule]]` | `time` | string `"HH:MM:SS"` | — |
| | `ring` | `start`/`end`/`special`/`custom`/`none` | — |
| | `custom_file` | string (optional) | — |

## Architecture notes

- **Single-instance** via `single_instance` crate (GUID-based).
- **Layered window** uses `UpdateLayeredWindow` with 32-bit BGRA DIB for per-pixel alpha: background semi-transparent, text fully opaque. Post-processing scans for non-background pixels and sets alpha=255.
- **Message loop** uses `MsgWaitForMultipleObjects` with 500ms timeout + `PeekMessageW`/`DispatchMessageW`.
- **GUI timer** fires every 500ms to refresh the time display and check auto-hide timeout.
- **Slide animation** via `AnimateWindow` (direction determined by window position relative to screen center).
- **Audio** uses `PlaySoundW` with `SND_MEMORY | SND_ASYNC` for embedded WAVs and `SND_FILENAME | SND_ASYNC` for custom WAVs. Volume via `waveOutSetVolume`.
- **Hotkey** uses `RegisterHotKey` / `UnregisterHotKey`; updated on config reload.
- **Auto-start** writes to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **i18n** detects system language via `GetUserDefaultLocaleName`; supports EN and ZH for tray menu, tooltip, and config template.
- Tray **left-click** toggles clock window; **right-click** opens context menu.
- "Skip next" increments an atomic counter; when the next scheduled second arrives, the counter is decremented instead of playing audio.
- Schedule matching is second-granularity (`entries_at` uses `HH:MM:SS` total seconds).

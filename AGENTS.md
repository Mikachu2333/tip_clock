# AGENTS.md

## Build & toolchain

- **Edition 2024** — stable Rust 1.85+.
- Windows-only. Uses `user32`, `kernel32`, `winmm`, `shcore`, `advapi32`, `comdlg32` FFI directly.
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
│                  auto-start registry, ChooseFont/ChooseColor dialogs
├── config.rs    — TOML config (auto-create, CJK punctuation auto-correct,
│                  time normalizer, i18n config templates)
├── audio.rs     — PlaySoundW (embedded + custom WAVs), waveOutSetVolume
├── gui.rs       — layered clock window (WS_EX_LAYERED, 32-bit BGRA per-pixel alpha,
│                  GDI text rendering, right-click popup menu)
├── hotkey.rs    — global hotkey via SetWindowsHookEx (WH_KEYBOARD_LL), not RegisterHotKey
└── i18n.rs      — GetUserDefaultLocaleName detection (EN/ZH), tray menu + config template i18n
res/
├── start.wav    — embedded via include_bytes!
├── end.wav
└── special.wav
config.toml      — generated on first run in EXE directory
```

## Configuration (`config.toml`)

- Read **once at startup** from EXE directory (not CWD). No hot-reload; restart to apply changes.
- Auto-created on first run with a language-aware template (ZH or EN).
- Auto-corrects:
  - CJK full-width punctuation (`：`→`:`, `＝`→`=`, `，`→`,`, etc.)
  - Bare time values (`time = 09:00` → `time = "09:00"`)
  - Time formats (`9` → `09:00:00`, `930` → `09:30:00`, `090000` → `09:00:00`)
- Missing keys filled from `GeneralConfig::default()`.
- Unparseable schedule entries logged and dropped.

### Config sections

| Section | Key | Type | Default |
|---------|-----|------|---------|
| `[general]` | `auto_start` | bool | `false` |
| | `bg_r` / `bg_g` / `bg_b` | u8 (0–255) | `0, 0, 0` |
| | `bg_opacity` | u8 (0–100) | `80` |
| | `text_r` / `text_g` / `text_b` | u8 (0–255) | `255, 255, 255` |
| | `font_name` | string | `"微软雅黑"` |
| | `font_size` | i32 | `16` |
| | `display_time` | u32 (1–60 s) | `5` |
| | `volume` | u8 (0–100) | `80` |
| | `hotkey_mod` | string | `"ctrl+alt"` |
| | `hotkey_key` | string | `"T"` |
| `[[schedule]]` | `time` | `"HH:MM:SS"` | — |
| | `ring` | `start`/`end`/`special`/`custom`/`none` | — |
| | `custom_file` | string (optional) | — |

## Architecture notes

- **Single-instance** via `single_instance` crate (GUID).
- **Layered window**: `UpdateLayeredWindow` + 32-bit BGRA DIB. Background pixels get configurable alpha; text pixels are post-processed to full opacity (alpha=255).
- **Message loop**: `MsgWaitForMultipleObjects` (500ms timeout) + `PeekMessageW` / `DispatchMessageW`.
- **GUI timer**: `WM_TIMER` every 500ms → redraws time, checks auto-hide timeout.
- **Hotkey**: `SetWindowsHookEx(WH_KEYBOARD_LL)` global hook. `RegisterHotKey` was dropped because it silently fails with `MOD_WIN` on some Windows versions. Callback compares VK + modifier state, posts `WM_USER_HOTKEY` to the clock window.
- **Audio**: `PlaySoundW` with `SND_MEMORY | SND_ASYNC` (embedded) or `SND_FILENAME | SND_ASYNC` (custom). Volume via `waveOutSetVolume`.
- **Auto-start**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` entry.
- **i18n**: `GetUserDefaultLocaleName` → EN / ZH. Tray menu, tooltip, and config template follow system language.
- **Tray icon**: left-click → toggle clock (`set_show_menu_on_left_click(false)` + `TrayIconEvent`). Right-click → context menu. Icon color: `rgb(61, 176, 87)`.
- **Clock window right-click**: Win32 popup menu (Hide / Exit).
- **Font & color**: `ChooseFontW` / `ChooseColorW` dialogs via tray menu. Persisted to `config.toml` on each change.
- **Skip next**: atomic counter; next schedule match decrements instead of playing.
- **Debug logging**: controlled by `cfg!(debug_assertions)`; `[main]` / `[gui]` / `[hotkey]` prefixes. Compiles to no-ops in release.

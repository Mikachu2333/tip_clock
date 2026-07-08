# AGENTS.md

## Build & toolchain
- **Edition 2024** — requires nightly Rust (`rustup default nightly` or `cargo +nightly build`).
- Windows-only. Uses `user32`, `kernel32`, `winmm`, `shcore` FFI directly.

```bash
cargo build --release    # release: LTO, single-cgu, stripped, panic=abort, no console window
cargo build              # debug: console window visible, slower
```

## Project structure
- `src/main.rs` — tray icon, Win32 message pump, main loop
- `src/config.rs` — TOML schedule parsing, `config.toml` load/create
- `src/audio.rs` — `PlaySoundW`-based WAV playback (WAVs embedded via `include_bytes!`)
- `res/*.wav` — audio assets compiled into binary; must exist at build time
- `res/*.mscz` — MuseScore source files (not used by code, only for editing)

## Configuration
- `config.toml` is read from **the EXE's directory** (not CWD).
- Auto-created on first run with a default schedule.
- Full-width colons (`：`) in time strings are auto-corrected to `:`.
- Entries with unparseable times are silently dropped.

## Architecture notes
- Single-instance enforced via `single_instance` crate (GUID-based).
- Timer loop uses `MsgWaitForMultipleObjects` with a 15 s timeout (not `Sleep`).
- Menu refresh is driven by a `NEED_REFRESH` atomic flag, set by skip/pause actions.
- "Skip next" increments a counter; when the next scheduled minute arrives, the counter is decremented instead of playing audio.
- No test suite, no CI, no linter config.

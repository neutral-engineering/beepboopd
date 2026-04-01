# beepboopd

Hourly chime daemon with multiple musical styles. Born April 1st, 2026.

## Build & check

```sh
cargo build          # debug build
cargo build --release
cargo clippy         # lint
cargo fmt            # format
cargo check          # type-check without building
```

## Test sounds

```sh
make beep            # OG success + failure
make clock           # Westminster chime at current hour
make chords          # chord at current hour
make scale           # scale run at current hour
make zelda           # zelda song for current hour
make h00..h23        # chords for specific hour
```

## Architecture

- `src/main.rs` — CLI (clap), logging (tracing), dispatch, systemd entry point
- `src/tunes.rs` — all audio: types (`BeepPattern`, `ZeldaSong`), sample rendering (`Buf`), play functions
- Audio is pre-rendered into a single `Vec<f32>` buffer per tune (no source transitions = no clicks)
- Phase is tracked continuously across notes for clean frequency changes
- All tones go through `sine_lp()` (low-pass filtered) for consistent warmth

## Styles

- `beep` — success/failure (default, even/odd hour)
- `clock` — Westminster chime + bongs (12h count)
- `chords` — major/minor by hour (24 unique)
- `scale` — ascending 5-note run (24 unique)
- `zelda` — 12 Ocarina of Time songs mapped to hours

## Config

- Volume: `-v <float>` or `BEEPBOOPD_VOLUME` env (default 0.9, range 0.0-1.0 for clean signal)
- Tune: `BEEPBOOPD_TUNE` env (beep, clock, chords, scale, zelda)
- Weekly: `BEEPBOOPD_WEEK` env (e.g. `m:zelda;t:chords;w:clock;th:beep;f:zelda;s:scale;su:clock`)
- WEEK overrides TUNE for specified days; unspecified days fall back to TUNE
- Logs: structured JSON to stderr via tracing, suppress with `BEEPBOOPD_LOG=false`
- Systemd: user service + timer in repo root, install via `make install`

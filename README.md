# beepboopd

april 1st, 2026

**beepboopd** (Beep Boop Daemon) is a tiny Rust program that plays a unique musical tune every hour

## Dependencies

Build: Rust 1.85+ and ALSA dev headers

```sh
# arch
sudo pacman -S alsa-lib

# debian/ubuntu
sudo apt install libasound2-dev
```

Runtime: PipeWire, PulseAudio, or ALSA for audio output.

## Install

```sh
make install
```

That's it. Your computer now chimes every hour.

To uninstall:

```sh
make uninstall
```

## Styles

beepboopd speaks five musical languages:

### `beep`

The original. A tribute to the bash script that started it all. Ascending tones for `success`, descending for `failure`. Even hours get success, odd hours get failure.

### `clock`

Westminster chime -- the classic "ding dong ding dong... dong dong ding dong" melody in 3/4 time, followed by deep bongs counting the hour (12h style, so midnight and noon both get 12).

### `chords`

Each hour gets its own chord -- major chords before noon, minor chords after. 24 unique chords walking the chromatic scale from C. Midnight is C major. High noon is C minor.

### `scale`

A quick five-note ascending run for each hour. Major scales for hours 0-11, minor for 12-23. The last note is held longer.

### `zelda`

12 Ocarina of Time songs, each mapped to two hours (H and H+12). Sun's Song plays at 07:00 and 19:00. Nocturne of Shadow at 01:00 and 13:00. All five ocarina notes (D4, F4, A4, B4, D5) with per-song timing.

Available songs: `lullaby`, `epona`, `saria`, `sun`, `time`, `storms`, `minuet`, `bolero`, `serenade`, `nocturne`, `requiem`, `prelude`.

## Usage

```sh
beepboopd                    # default: beep success/failure by hour
beepboopd run                # daemon mode: chime every hour
beepboopd clock              # Westminster chime at current hour
beepboopd chords 15          # 3 PM's chord (C# minor)
beepboopd scale 7            # 7 AM's scale run
beepboopd beep success       # the OG ascending beep
beepboopd zelda storms       # Song of Storms
beepboopd zelda              # picks song by current hour
beepboopd -v 0.5 clock       # quieter
```

## Configuration

| Env var            | Default | Purpose                                         |
| ------------------ | ------- | ----------------------------------------------- |
| `BEEPBOOPD_VOLUME` | `0.9`   | Amplitude multiplier (0.0-1.0 for clean signal) |
| `BEEPBOOPD_TUNE`   | `beep`  | Default style when no subcommand given          |
| `BEEPBOOPD_WEEK`   | --      | Per-day style schedule (see below)              |
| `BEEPBOOPD_LOG`    | --      | Set to `false` or `0` to suppress info logs     |

`BEEPBOOPD_WEEK` overrides `BEEPBOOPD_TUNE` for days that are specified. Unspecified days fall back to `BEEPBOOPD_TUNE`, then to `beep`.

Volume can also be set with `-v <float>` on the command line.

### Weekly schedule

Set a different style for each day of the week:

```sh
export BEEPBOOPD_WEEK="m:zelda;t:chords;w:clock;th:beep;f:zelda;s:scale;su:clock"
```

Day keys: `su` `m` `t` `w` `th` `f` `s`

## Testing

```sh
make beep     # OG success + failure
make clock    # Westminster chime at current hour
make chords   # chord at current hour
make scale    # scale run at current hour
make zelda    # zelda song for current hour
make all      # play all styles in sequence
make h00      # chords for specific hour (h00-h23)
```

## The hour-chord map

| Hour | Root | Quality | Hour | Root | Quality |
| ---- | ---- | ------- | ---- | ---- | ------- |
| 0    | C    | major   | 12   | C    | minor   |
| 1    | C#   | major   | 13   | C#   | minor   |
| 2    | D    | major   | 14   | D    | minor   |
| 3    | D#   | major   | 15   | D#   | minor   |
| 4    | E    | major   | 16   | E    | minor   |
| 5    | F    | major   | 17   | F    | minor   |
| 6    | F#   | major   | 18   | F#   | minor   |
| 7    | G    | major   | 19   | G    | minor   |
| 8    | G#   | major   | 20   | G#   | minor   |
| 9    | A    | major   | 21   | A    | minor   |
| 10   | A#   | major   | 22   | A#   | minor   |
| 11   | B    | major   | 23   | B    | minor   |

## The zelda hour map

| Hour | Song               | Hour | Song               |
| ---- | ------------------ | ---- | ------------------ |
| 0    | Song of Time       | 12   | Song of Time       |
| 1    | Nocturne of Shadow | 13   | Nocturne of Shadow |
| 2    | Requiem of Spirit  | 14   | Requiem of Spirit  |
| 3    | Zelda's Lullaby    | 15   | Zelda's Lullaby    |
| 4    | Serenade of Water  | 16   | Serenade of Water  |
| 5    | Minuet of Forest   | 17   | Minuet of Forest   |
| 6    | Prelude of Light   | 18   | Prelude of Light   |
| 7    | Sun's Song         | 19   | Sun's Song         |
| 8    | Epona's Song       | 20   | Epona's Song       |
| 9    | Saria's Song       | 21   | Saria's Song       |
| 10   | Bolero of Fire     | 22   | Bolero of Fire     |
| 11   | Song of Storms     | 23   | Song of Storms     |

## Logging

Structured JSON to stderr via `tracing`. Every play emits:

```json
{
  "timestamp": "...",
  "level": "INFO",
  "fields": {
    "message": "playing",
    "cmd": "zelda",
    "tune": "storms",
    "volume": 0.9
  }
}
```

Suppress with `BEEPBOOPD_LOG=false`. Errors always log.

When running under systemd, logs land in the journal: `journalctl --user -u beepboopd`.

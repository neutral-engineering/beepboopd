mod tunes;

use clap::{Parser, Subcommand, ValueEnum};
use rodio::{DeviceSinkBuilder, Player};
use std::fmt;
use tracing::info;
use tunes::{BeepPattern, Tune, ZELDA_BY_HOUR, ZeldaSong};

#[derive(Parser)]
#[command(name = "beepboopd", about = "the beep boop daemon", version)]
struct Cli {
    /// Volume multiplier
    #[arg(short, long, env = "BEEPBOOPD_VOLUME", default_value_t = 0.9)]
    volume: f32,

    /// BPM override (uses each tune's default if not set)
    #[arg(short, long, env = "BEEPBOOPD_BPM")]
    bpm: Option<f32>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// The OG beep (even hours: success, odd hours: failure)
    Beep { pattern: Option<BeepPattern> },
    /// Westminster clock chimes then bongs for the hour
    Clock {
        #[arg(value_parser = clap::value_parser!(u32).range(0..24))]
        hour: Option<u32>,
    },
    /// Unique chord per hour: major before noon, minor after
    Chords {
        #[arg(value_parser = clap::value_parser!(u32).range(0..24))]
        hour: Option<u32>,
    },
    /// Ascending scale run: major AM, minor PM
    Scale {
        #[arg(value_parser = clap::value_parser!(u32).range(0..24))]
        hour: Option<u32>,
    },
    /// Ocarina of Time songs (picks by hour if no song given)
    Zelda { song: Option<ZeldaSong> },
    /// The Lick — jazz's most famous phrase in all 12 keys
    Jazz {
        #[arg(value_parser = clap::value_parser!(u32).range(0..24))]
        hour: Option<u32>,
    },
    /// Run as a daemon: chime every hour
    Run,
    /// Install systemd user service
    Install,
    /// Uninstall systemd user service
    Uninstall,
    /// Show systemd service status + recent logs
    Status,
}

fn log_play(cmd: &str, tune: &dyn fmt::Display, volume: f32) {
    info!(cmd = cmd, tune = %tune, volume = volume, "playing");
}

/// Resolve the current tune and play it.
fn play_now(vol: f32, bpm: Option<f32>) {
    let tune = tune_for_today()
        .or_else(|| {
            std::env::var("BEEPBOOPD_TUNE")
                .ok()
                .and_then(|s| Tune::from_str(&s, true).ok())
        })
        .unwrap_or(Tune::Beep);

    let mut h = DeviceSinkBuilder::open_default_sink().expect("Failed to open audio output");
    h.log_on_drop(false);
    let player = Player::connect_new(h.mixer());

    match tune {
        Tune::Beep => {
            let pattern = if current_hour().is_multiple_of(2) {
                BeepPattern::Success
            } else {
                BeepPattern::Failure
            };
            log_play("beep", &pattern, vol);
            tunes::play_beep(&player, vol, bpm, &pattern);
        }
        Tune::Zelda => {
            let song = ZELDA_BY_HOUR[(current_hour() % 12) as usize];
            log_play("zelda", &song, vol);
            tunes::play_zelda(&player, vol, bpm, &song);
        }
        Tune::Clock => {
            let hour = current_hour();
            log_play("clock", &hour, vol);
            tunes::play_clock(&player, vol, bpm, hour);
        }
        Tune::Chords => {
            let hour = current_hour();
            log_play("chords", &hour, vol);
            tunes::play_chords(&player, vol, bpm, hour);
        }
        Tune::Scale => {
            let hour = current_hour();
            log_play("scale", &hour, vol);
            tunes::play_scale(&player, vol, bpm, hour);
        }
        Tune::Jazz => {
            let hour = current_hour();
            log_play("jazz", &hour, vol);
            tunes::play_jazz(&player, vol, bpm, hour);
        }
    }

    player.sleep_until_end();
}

/// Seconds until the next chime boundary for a given period in minutes.
fn secs_until_next_chime(period_min: u64) -> u64 {
    let tm = local_time();
    let elapsed = tm.tm_min as u64 * 60 + tm.tm_sec as u64;
    let period_secs = period_min * 60;
    let remaining = period_secs - (elapsed % period_secs);
    if remaining == 0 {
        period_secs
    } else {
        remaining
    }
}

/// Get current local time.
fn local_time() -> libc::tm {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm = std::mem::zeroed::<libc::tm>();
        libc::localtime_r(&t, &mut tm);
        tm
    }
}

/// Current Unix epoch time in seconds.
fn epoch_secs() -> i64 {
    unsafe { libc::time(std::ptr::null_mut()) }
}

/// Sleep until an absolute wall-clock time (Unix epoch seconds).
///
/// On Linux, uses `clock_nanosleep(CLOCK_REALTIME, TIMER_ABSTIME)` which
/// returns immediately after system suspend if the target time has passed.
/// On other platforms, polls with short relative sleeps.
fn sleep_until_epoch(target: i64) {
    #[cfg(target_os = "linux")]
    unsafe {
        let ts = libc::timespec {
            tv_sec: target as libc::time_t,
            tv_nsec: 0,
        };
        libc::clock_nanosleep(
            libc::CLOCK_REALTIME,
            libc::TIMER_ABSTIME,
            &ts,
            std::ptr::null_mut(),
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        loop {
            let now = epoch_secs();
            if now >= target {
                break;
            }
            let delta = (target - now).min(10) as u64;
            std::thread::sleep(std::time::Duration::from_secs(delta));
        }
    }
}

fn current_hour() -> u32 {
    local_time().tm_hour as u32
}

/// Current day of week: 0=Sunday, 1=Monday, ..., 6=Saturday.
fn current_wday() -> u32 {
    local_time().tm_wday as u32
}

/// Parse BEEPBOOPD_WEEK="m:zelda;t:chords;w:clock;th:beep;f:zelda;s:zelda;su:clock"
fn tune_for_today() -> Option<Tune> {
    let week = std::env::var("BEEPBOOPD_WEEK").ok()?;
    let wday = current_wday();
    let day_key = match wday {
        0 => "su",
        1 => "m",
        2 => "t",
        3 => "w",
        4 => "th",
        5 => "f",
        6 => "s",
        _ => return None,
    };

    for entry in week.split(';') {
        let (key, val) = entry.split_once(':')?;
        if key == day_key {
            return Tune::from_str(val, true).ok();
        }
    }
    None
}

const SYSTEMD_TEMPLATE: &str = include_str!("../beepboopd.service");
const LAUNCHD_TEMPLATE: &str = include_str!("../engineering.neutral.beepboopd.plist");

fn home_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").expect("HOME not set"))
}

fn exe_path() -> String {
    std::env::current_exe()
        .expect("could not determine binary path")
        .to_string_lossy()
        .into_owned()
}

const ENV_KEYS: &[&str] = &[
    "BEEPBOOPD_TUNE",
    "BEEPBOOPD_VOLUME",
    "BEEPBOOPD_WEEK",
    "BEEPBOOPD_BPM",
    "BEEPBOOPD_PERIOD_MINUTES",
    "BEEPBOOPD_LOG",
];

fn systemd_env_lines() -> String {
    let mut lines = String::new();
    for &key in ENV_KEYS {
        if let Ok(val) = std::env::var(key) {
            lines.push_str(&format!("Environment=\"{key}={val}\"\n"));
        }
    }
    lines
}

fn launchd_env_dict() -> String {
    let mut entries = String::new();
    for &key in ENV_KEYS {
        if let Ok(val) = std::env::var(key) {
            entries.push_str(&format!(
                "        <key>{key}</key>\n        <string>{val}</string>\n"
            ));
        }
    }
    if entries.is_empty() {
        return String::new();
    }
    format!("    <key>EnvironmentVariables</key>\n    <dict>\n{entries}    </dict>\n")
}

fn install_service() {
    let exe = exe_path();

    if cfg!(target_os = "macos") {
        let dir = home_dir().join("Library/LaunchAgents");
        std::fs::create_dir_all(&dir).expect("could not create LaunchAgents dir");
        let path = dir.join("engineering.neutral.beepboopd.plist");

        let _ = std::process::Command::new("launchctl")
            .args(["unload", &path.to_string_lossy()])
            .output();

        let plist = LAUNCHD_TEMPLATE.replace("EXEC_PATH", &exe).replace(
            "</dict>\n</plist>",
            &format!("{}</dict>\n</plist>", launchd_env_dict()),
        );
        std::fs::write(&path, plist).expect("could not write plist");
        eprintln!("wrote {}", path.display());

        let _ = std::process::Command::new("launchctl")
            .args(["load", &path.to_string_lossy()])
            .status();
    } else {
        let dir = home_dir().join(".config/systemd/user");
        std::fs::create_dir_all(&dir).expect("could not create systemd user dir");
        let path = dir.join("beepboopd.service");

        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", "beepboopd.service"])
            .output();

        let service = SYSTEMD_TEMPLATE
            .replace("EXEC_PATH", &exe)
            .replace("ExecStart=", &format!("{}ExecStart=", systemd_env_lines()));
        std::fs::write(&path, service).expect("could not write service file");
        eprintln!("wrote {}", path.display());

        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "enable", "--now", "beepboopd.service"])
            .status();
    }

    eprintln!("beepboopd installed and started");
}

fn uninstall_service() {
    if cfg!(target_os = "macos") {
        let path = home_dir().join("Library/LaunchAgents/engineering.neutral.beepboopd.plist");
        let _ = std::process::Command::new("launchctl")
            .args(["unload", &path.to_string_lossy()])
            .status();
        if path.exists() {
            std::fs::remove_file(&path).expect("could not remove plist");
            eprintln!("removed {}", path.display());
        }
    } else {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", "--now", "beepboopd.service"])
            .status();
        let path = home_dir().join(".config/systemd/user/beepboopd.service");
        if path.exists() {
            std::fs::remove_file(&path).expect("could not remove service file");
            eprintln!("removed {}", path.display());
        }
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
    }
    eprintln!("beepboopd uninstalled");
}

fn show_status() {
    if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("launchctl")
            .args(["list", "engineering.neutral.beepboopd"])
            .status();
        eprintln!("\nlogs: /tmp/beepboopd.log");
        let _ = std::process::Command::new("tail")
            .args(["-n", "10", "/tmp/beepboopd.log"])
            .status();
    } else {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "status", "beepboopd.service"])
            .status();
        eprintln!();
        let _ = std::process::Command::new("journalctl")
            .args([
                "--user",
                "-u",
                "beepboopd.service",
                "-n",
                "10",
                "--no-pager",
            ])
            .status();
    }
}

fn main() {
    let log_level = match std::env::var("BEEPBOOPD_LOG").ok().as_deref() {
        Some("false" | "0") => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .init();

    let cli_vol: Option<f32> = std::env::var("BEEPBOOPD_VOLUME")
        .ok()
        .and_then(|v| v.parse().ok());
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            if (e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion)
                && let Ok(mut h) = DeviceSinkBuilder::open_default_sink()
            {
                h.log_on_drop(false);
                let p = Player::connect_new(h.mixer());
                tunes::play_wee_woo(&p, cli_vol.unwrap_or(0.9) / 2.0);
                p.sleep_until_end();
            }
            std::process::exit(e.exit_code());
        }
    };
    let vol = cli.volume;
    let bpm = cli.bpm;

    match cli.command.unwrap_or(Command::Beep { pattern: None }) {
        Command::Install => install_service(),
        Command::Uninstall => uninstall_service(),
        Command::Status => show_status(),
        Command::Run => {
            let period_min: u64 = std::env::var("BEEPBOOPD_PERIOD_MINUTES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            let tune_env = std::env::var("BEEPBOOPD_TUNE").ok();
            let week_env = std::env::var("BEEPBOOPD_WEEK").ok();
            info!(
                volume = vol,
                tune = tune_env.as_deref().unwrap_or("beep"),
                week = week_env.as_deref().unwrap_or(""),
                period_min = period_min,
                "started"
            );

            let period_secs = period_min * 60;
            let mut last_chime = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(period_secs))
                .unwrap_or_else(std::time::Instant::now);
            loop {
                let until_chime = secs_until_next_chime(period_min);
                let target = epoch_secs() + until_chime as i64;
                sleep_until_epoch(target);

                let since_last = last_chime.elapsed().as_secs();
                let remaining = secs_until_next_chime(period_min);

                // Normal: we woke right at the boundary.
                // Sleep/wake: we overslept past it — enough real time
                // has elapsed that we know we missed a boundary.
                let at_boundary = remaining <= 5 || remaining >= period_secs - 5;
                let missed = since_last >= period_secs - 5;

                if (at_boundary || missed) && since_last > period_secs / 2 {
                    play_now(vol, bpm);
                    last_chime = std::time::Instant::now();
                }
            }
        }
        cmd => {
            let mut handle =
                DeviceSinkBuilder::open_default_sink().expect("Failed to open audio output");
            handle.log_on_drop(false);
            let player = Player::connect_new(handle.mixer());

            // Priority: subcommand > BEEPBOOPD_WEEK (today) > BEEPBOOPD_TUNE (fallback) > beep
            let default_tune = tune_for_today()
                .or_else(|| {
                    std::env::var("BEEPBOOPD_TUNE")
                        .ok()
                        .and_then(|s| Tune::from_str(&s, true).ok())
                })
                .unwrap_or(Tune::Beep);

            let cmd = match cmd {
                Command::Beep { pattern: None } => match default_tune {
                    Tune::Beep => Command::Beep { pattern: None },
                    Tune::Clock => Command::Clock { hour: None },
                    Tune::Chords => Command::Chords { hour: None },
                    Tune::Scale => Command::Scale { hour: None },
                    Tune::Zelda => Command::Zelda { song: None },
                    Tune::Jazz => Command::Jazz { hour: None },
                },
                other => other,
            };

            match cmd {
                Command::Beep { pattern } => {
                    let pattern = pattern.unwrap_or_else(|| {
                        if current_hour().is_multiple_of(2) {
                            BeepPattern::Success
                        } else {
                            BeepPattern::Failure
                        }
                    });
                    log_play("beep", &pattern, vol);
                    tunes::play_beep(&player, vol, bpm, &pattern);
                }
                Command::Zelda { song } => {
                    let song =
                        song.unwrap_or_else(|| ZELDA_BY_HOUR[(current_hour() % 12) as usize]);
                    log_play("zelda", &song, vol);
                    tunes::play_zelda(&player, vol, bpm, &song);
                }
                Command::Clock { hour } => {
                    let hour = hour.unwrap_or_else(current_hour);
                    log_play("clock", &hour, vol);
                    tunes::play_clock(&player, vol, bpm, hour);
                }
                Command::Chords { hour } => {
                    let hour = hour.unwrap_or_else(current_hour);
                    log_play("chords", &hour, vol);
                    tunes::play_chords(&player, vol, bpm, hour);
                }
                Command::Scale { hour } => {
                    let hour = hour.unwrap_or_else(current_hour);
                    log_play("scale", &hour, vol);
                    tunes::play_scale(&player, vol, bpm, hour);
                }
                Command::Jazz { hour } => {
                    let hour = hour.unwrap_or_else(current_hour);
                    log_play("jazz", &hour, vol);
                    tunes::play_jazz(&player, vol, bpm, hour);
                }
                Command::Run | Command::Install | Command::Uninstall | Command::Status => {
                    unreachable!()
                }
            }

            player.sleep_until_end();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_template_has_exec_placeholder() {
        assert!(SYSTEMD_TEMPLATE.contains("EXEC_PATH"));
    }

    #[test]
    fn systemd_template_substitution() {
        let result = SYSTEMD_TEMPLATE.replace("EXEC_PATH", "/usr/local/bin/beepboopd");
        assert!(result.contains("ExecStart=/usr/local/bin/beepboopd run"));
        assert!(!result.contains("EXEC_PATH"));
    }

    #[test]
    fn launchd_template_has_exec_placeholder() {
        assert!(LAUNCHD_TEMPLATE.contains("EXEC_PATH"));
    }

    #[test]
    fn launchd_template_is_valid_xml() {
        let result = LAUNCHD_TEMPLATE.replace("EXEC_PATH", "/usr/local/bin/beepboopd");
        assert!(!result.contains("EXEC_PATH"));
        assert!(result.contains("engineering.neutral.beepboopd"));
        assert!(result.starts_with("<?xml"));
        // Verify balanced tags
        assert_eq!(
            result.matches("<dict>").count(),
            result.matches("</dict>").count()
        );
        assert_eq!(
            result.matches("<array>").count(),
            result.matches("</array>").count()
        );
        assert!(result.contains("</plist>"));
    }

    #[test]
    fn launchd_template_with_env_injection() {
        let result = LAUNCHD_TEMPLATE
            .replace("EXEC_PATH", "/usr/local/bin/beepboopd")
            .replace(
                "</dict>\n</plist>",
                "    <key>EnvironmentVariables</key>\n    <dict>\n        <key>BEEPBOOPD_TUNE</key>\n        <string>zelda</string>\n    </dict>\n</dict>\n</plist>",
            );
        assert!(result.contains("BEEPBOOPD_TUNE"));
        assert!(result.contains("zelda"));
        assert_eq!(
            result.matches("<dict>").count(),
            result.matches("</dict>").count()
        );
    }

    #[test]
    fn systemd_env_lines_from_env() {
        // SAFETY: test runs single-threaded, no concurrent env access
        unsafe { std::env::set_var("BEEPBOOPD_TUNE", "zelda") };
        let lines = systemd_env_lines();
        assert!(lines.contains("BEEPBOOPD_TUNE=zelda"));
        unsafe { std::env::remove_var("BEEPBOOPD_TUNE") };
    }
}

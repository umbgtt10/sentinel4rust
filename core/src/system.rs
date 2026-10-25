// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! System implementations of the small contracts: processes, the clock, sleeping, the journal.

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Child, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::model::timestamp::Timestamp;
use crate::traits::{Clock, Command, CommandResult, CommandRunner, Journal, Pause};

/// Runs commands with `std::process`, each in its own process group so a timeout kills its children too.
#[derive(Debug, Default)]
pub struct SystemCommandRunner;

const POLL: Duration = Duration::from_millis(20);

fn reader(stream: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

fn kill_group(child: &mut Child) {
    // The child leads its own group (process_group(0)), so its pid is the group id.
    let _ = std::process::Command::new("kill")
        .args(["-KILL", "--", &format!("-{}", child.id())])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

impl CommandRunner for SystemCommandRunner {
    fn run(&self, command: &Command) -> CommandResult {
        let spawned = std::process::Command::new(&command.program)
            .args(&command.args)
            .current_dir(&command.dir)
            .envs(command.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => return CommandResult::FailedToStart(error.to_string()),
        };
        let stdout = reader(child.stdout.take());
        let stderr = reader(child.stderr.take());
        let deadline = Instant::now() + command.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() >= deadline => {
                    kill_group(&mut child);
                    let _ = child.wait();
                    break None;
                }
                Ok(None) => thread::sleep(POLL),
                Err(error) => {
                    kill_group(&mut child);
                    let _ = child.wait();
                    return CommandResult::FailedToStart(error.to_string());
                }
            }
        };
        let stdout = stdout.join().unwrap_or_default();
        let stderr = stderr.join().unwrap_or_default();
        match status {
            Some(status) => CommandResult::Exited {
                code: status.code(),
                stdout,
                stderr,
            },
            None => CommandResult::TimedOut { stdout, stderr },
        }
    }
}

/// The system clock.
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Timestamp::from_unix(i64::try_from(seconds).unwrap_or(i64::MAX))
    }
}

/// Sleeps the current thread.
#[derive(Debug, Default)]
pub struct ThreadPause;

impl Pause for ThreadPause {
    fn pause(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

/// Writes journal lines to standard output (systemd adds the timestamp).
#[derive(Debug, Default)]
pub struct StdoutJournal;

impl Journal for StdoutJournal {
    fn line(&self, text: &str) {
        println!("{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(script: &str, timeout: Duration) -> CommandResult {
        let command = Command::new("sh", &["-c", script], &std::env::temp_dir(), timeout)
            .env("SENTINEL_TEST", "yes");
        SystemCommandRunner.run(&command)
    }

    #[test]
    fn captures_exit_code_output_and_environment() {
        let result = sh(
            "echo out $SENTINEL_TEST; echo err >&2; exit 3",
            Duration::from_secs(10),
        );
        assert_eq!(
            result,
            CommandResult::Exited {
                code: Some(3),
                stdout: "out yes\n".to_string(),
                stderr: "err\n".to_string(),
            }
        );
        assert_eq!(result.success(), None);
        assert_eq!(result.describe(), "exited with 3: err");
    }

    #[test]
    fn success_returns_stdout() {
        assert_eq!(
            sh("printf hi", Duration::from_secs(10)).success(),
            Some("hi")
        );
    }

    #[test]
    fn a_command_past_its_deadline_is_killed_with_its_children() {
        let marker = std::env::temp_dir().join(format!("sentinel-child-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let script = format!(
            "(sleep 2; touch {}) & echo started; sleep 30",
            marker.display()
        );
        let start = Instant::now();
        let result = sh(&script, Duration::from_millis(300));
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "{:?}",
            start.elapsed()
        );
        assert!(matches!(&result, CommandResult::TimedOut { stdout, .. } if stdout == "started\n"));
        thread::sleep(Duration::from_millis(2500));
        assert!(
            !marker.exists(),
            "the background child survived the timeout"
        );
    }

    #[test]
    fn a_missing_program_fails_to_start() {
        let command = Command::new(
            "/nonexistent/program",
            &[],
            &std::env::temp_dir(),
            Duration::from_secs(1),
        );
        assert!(matches!(
            SystemCommandRunner.run(&command),
            CommandResult::FailedToStart(_)
        ));
    }

    #[test]
    fn the_clock_is_after_2026_and_pause_waits() {
        assert!(SystemClock.now() > Timestamp::from_unix(1_767_225_600));
        let start = Instant::now();
        ThreadPause.pause(Duration::from_millis(30));
        assert!(start.elapsed() >= Duration::from_millis(30));
    }
}

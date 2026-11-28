// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serge4rust::process::command_outcome::CommandOutcome;
use serge4rust::process::command_runner::CommandRunner;
use serge4rust::process::launch_failure::LaunchFailure;
use std::cell::RefCell;

// The seam every gate is constructed against. Lives in the gate tests' support,
// so they share one fake rather than each growing their own.
pub struct FakeCommandRunner {
    available: bool,
    streaming_code: Option<i32>,
    capturing_code: Option<i32>,
    stdout: String,
    calls: RefCell<Vec<Vec<String>>>,
}

impl FakeCommandRunner {
    pub fn new() -> Self {
        Self {
            available: true,
            streaming_code: Some(0),
            capturing_code: Some(0),
            stdout: String::new(),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    // A captured run answers with the streaming code too, unless told otherwise:
    // a gate that captures one command and streams another needs the two apart.
    pub fn with_capturing_code(mut self, capturing_code: Option<i32>) -> Self {
        self.capturing_code = capturing_code;
        self
    }

    pub fn with_streaming_code(mut self, streaming_code: Option<i32>) -> Self {
        self.streaming_code = streaming_code;
        self.capturing_code = streaming_code;
        self
    }

    pub fn with_stdout(mut self, stdout: &str) -> Self {
        self.stdout = String::from(stdout);
        self
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }

    fn record(&self, args: &[String]) {
        self.calls.borrow_mut().push(args.to_vec());
    }
}

impl Default for FakeCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        self.available
    }

    fn run_streaming(&self, _program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure> {
        self.record(args);
        Ok(self.streaming_code)
    }

    fn run_capturing(
        &self,
        _program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        self.record(args);
        Ok(CommandOutcome::new(
            self.capturing_code,
            self.stdout.clone(),
            String::new(),
        ))
    }
}

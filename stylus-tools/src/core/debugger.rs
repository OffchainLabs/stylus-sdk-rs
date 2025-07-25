// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    ffi::OsStr,
    fmt::Display,
    process::{Command, Stdio},
};

use cfg_if::cfg_if;

use crate::utils::sys;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Debug)]
pub struct Debugger {
    pub program: &'static str,
    debugger_args: &'static [&'static str],
}

impl Debugger {
    fn new(program: &'static str, debugger_args: &'static [&'static str]) -> Self {
        Self {
            program,
            debugger_args,
        }
    }

    /// Select debugger depending on availability.
    ///
    /// The system is checked for the following, in this order:
    /// - rust-gdb (skipped on MacOS)
    /// - rust-lldb
    /// - gdb (skipped on MacOS)
    /// - lldb
    pub fn select() -> Result<Self, NoDebuggerFound> {
        let macos = cfg!(target_os = "macos");
        if sys::command_exists("rust-gdb") && !macos {
            Ok(Self::new("rust-gdb", GDB_ARGS))
        } else if sys::command_exists("rust-lldb") {
            Ok(Self::new("rust-lldb", LLDB_ARGS))
        } else {
            log::info!("rust specific debugger not installed, falling back to generic debugger");
            if sys::command_exists("gdb") && !macos {
                Ok(Self::new("gdb", GDB_ARGS))
            } else if sys::command_exists("lldb") {
                Ok(Self::new("lldb", LLDB_ARGS))
            } else {
                Err(NoDebuggerFound)
            }
        }
    }

    fn as_command(&self, program_args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Command {
        let mut command = Command::new(self.program);
        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .args(self.debugger_args)
            .args(program_args);
        command
    }

    pub fn exec(&self, program_args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> impl Display {
        let mut cmd = self.as_command(program_args);

        cfg_if! {
            if #[cfg(unix)] {
                cmd.exec()
            } else if #[cfg(windows)] {
                cmd.status()
            } else {
                unimplemented!("unsupported platform")
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("no debugger found")]
pub struct NoDebuggerFound;

const GDB_ARGS: &[&str] = &[
    "--quiet",
    "-ex=set breakpoint pending on",
    "-ex=b user_entrypoint",
    "-ex=r",
    "--args",
];

const LLDB_ARGS: &[&str] = &[
    "--source-quietly",
    "-o",
    "b user_entrypoint",
    "-o",
    "r",
    "--",
];

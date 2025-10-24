// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{bail, eyre};
use stylus_tools::{
    core::{build::BuildConfig, project::workspace::Workspace, tracing::Trace},
    utils::{color::Color, sys},
};

use crate::{
    common_args::{ProjectArgs, ProviderArgs, TraceArgs},
    utils::hostio,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Whether to use stable Rust. Note that nightly is needed to expand macros.
    #[arg(short, long)]
    stable_rust: bool,
    /// Any features that should be passed to cargo build.
    #[arg(short, long)]
    features: Option<Vec<String>>,
    /// Which specific package to build during replay, if any.
    #[arg(long)]
    package: Option<String>,
    /// Whether this process is the child of another.
    #[arg(short, long, hide(true))]
    child: bool,

    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    trace: TraceArgs,
}

pub async fn exec(args: Args) -> eyre::Result<()> {
    let mut contracts = args.project.contracts()?;
    if contracts.len() != 1 {
        bail!("cargo stylus trace can only be executed on one contract at a time");
    }
    let contract = contracts.pop().unwrap();

    let macos = cfg!(target_os = "macos");
    if !args.child {
        let gdb_args = [
            "--quiet",
            "-ex=set breakpoint pending on",
            "-ex=b user_entrypoint",
            "-ex=r",
            "--args",
        ]
        .as_slice();
        let lldb_args = [
            "--source-quietly",
            "-o",
            "b user_entrypoint",
            "-o",
            "r",
            "--",
        ]
        .as_slice();
        let (cmd_name, args) = if sys::command_exists("rust-gdb") && !macos {
            ("rust-gdb", &gdb_args)
        } else if sys::command_exists("rust-lldb") {
            ("rust-lldb", &lldb_args)
        } else {
            println!("rust specific debugger not installed, falling back to generic debugger");
            if sys::command_exists("gdb") && !macos {
                ("gdb", &gdb_args)
            } else if sys::command_exists("lldb") {
                ("lldb", &lldb_args)
            } else {
                bail!("no debugger found")
            }
        };
        let mut cmd = Command::new(cmd_name);
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        for arg in args.iter() {
            cmd.arg(arg);
        }

        for arg in std::env::args() {
            cmd.arg(arg);
        }
        cmd.arg("--child");

        #[cfg(unix)]
        let err = cmd.exec();
        #[cfg(windows)]
        let err = cmd.status();

        bail!("failed to exec {cmd_name} {:?}", err);
    }

    let provider = args.provider.build_provider().await?;

    let trace = Trace::new(args.trace.tx, &args.trace.config, &provider).await?;

    let config = BuildConfig {
        features: args.features.clone().unwrap_or_default(),
        ..Default::default()
    };
    let _wasm = contract.build(&config)?;

    build_shared_library(&args.trace.project, args.package, args.features)?;
    let target_dir = Workspace::current()?.metadata.target_directory;
    let library_extension = if macos { ".dylib" } else { ".so" };
    let shared_library = find_shared_library(target_dir.as_ref(), library_extension)?;

    // TODO: don't assume the contract is top-level
    let Some(args) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = args.len();

    unsafe {
        *hostio::FRAME.lock() = Some(trace.reader());

        type Entrypoint = unsafe extern "C" fn(usize) -> usize;
        let lib = libloading::Library::new(shared_library)?;
        let main: libloading::Symbol<Entrypoint> = lib.get(b"user_entrypoint")?;

        match main(args_len) {
            0 => println!("call completed successfully"),
            1 => println!("call reverted"),
            x => println!("call exited with unknown status code: {}", x.red()),
        }
    }
    Ok(())
}

pub fn build_shared_library(
    path: &Path,
    package: Option<String>,
    features: Option<Vec<String>>,
) -> eyre::Result<()> {
    let mut cargo = sys::new_command("cargo");

    cargo.current_dir(path).arg("build");

    if let Some(f) = features {
        cargo.arg("--features").arg(f.join(","));
    }
    if let Some(p) = package {
        cargo.arg("--package").arg(p);
    }

    cargo
        .arg("--lib")
        .arg("--locked")
        .arg("--target")
        .arg(rustc_host::from_cli()?)
        .output()?;
    Ok(())
}

pub fn find_shared_library(project: &Path, extension: &str) -> eyre::Result<PathBuf> {
    let triple = rustc_host::from_cli()?;
    let so_dir = project.join(format!("{triple}/debug/"));
    let so_dir = std::fs::read_dir(&so_dir)
        .map_err(|e| eyre!("failed to open {}: {e}", so_dir.to_string_lossy()))?
        .filter_map(|r| r.ok())
        .map(|r| r.path())
        .filter(|r| r.is_file());

    let mut file: Option<PathBuf> = None;
    for entry in so_dir {
        let Some(ext) = entry.file_name() else {
            continue;
        };
        let ext = ext.to_string_lossy();

        if ext.contains(extension) {
            if let Some(other) = file {
                let other = other.file_name().unwrap().to_string_lossy();
                bail!("more than one .so found: {ext} and {other}",);
            }
            file = Some(entry);
        }
    }
    let Some(file) = file else {
        bail!("failed to find .so");
    };
    Ok(file)
}

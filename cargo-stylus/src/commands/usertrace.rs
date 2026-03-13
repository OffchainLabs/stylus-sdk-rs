// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! User-level trace command for Stylus contracts
//!
//! This command replays a transaction under `rust-stylusdb` with crate-filtered
//! call tracing to visualize user function calls in Stylus contracts.
//! The parent process spawns `rust-stylusdb` (an LLDB-based debugger), which
//! re-invokes this command with `--child` to replay the transaction under
//! call-tracing instrumentation.

use alloy::providers::Provider;
use eyre::{bail, Context};
use std::{
    path::Path,
    process::{Command, Stdio},
};
use stylus_tools::{
    core::{build::BuildConfig, project::workspace::Workspace, tracing::Trace},
    utils::sys,
};

use crate::{
    commands::replay::find_shared_library,
    common_args::{ProjectArgs, ProviderArgs, TraceArgs},
    error::CargoStylusResult,
    utils::hostio,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    trace: TraceArgs,

    /// Any features that should be passed to cargo build.
    #[arg(short, long)]
    features: Option<Vec<String>>,

    /// Which specific package to build during trace, if any.
    #[arg(long)]
    package: Option<String>,

    /// Whether this process is the child of another.
    #[arg(short, long, hide(true))]
    child: bool,

    /// Contract addresses and their source paths for multi-contract tracing.
    /// Format: ADDRESS1:PATH1,ADDRESS2:PATH2,...
    /// Example: 0x123...:./contractA,0x456...:./contractB
    #[arg(long, value_delimiter = ',', value_name = "CONTRACTS")]
    contracts: Option<Vec<String>>,

    /// Include stylus_sdk functions in trace
    #[arg(long)]
    verbose_usertrace: bool,

    /// Comma-separated list of other crates to trace
    /// Example: --trace-external-usertrace="std,core,other_contract"
    #[arg(long, value_delimiter = ',')]
    trace_external_usertrace: Vec<String>,

    /// If passed, do NOT redirect stylusdb's output to `/dev/null`
    /// By default, we silence stylusdb to keep console output clean
    #[arg(long, default_value_t = false)]
    enable_stylusdb_output: bool,
}

/// Derive the crate name from a shared library path by stripping the
/// conventional `lib` prefix from the file stem (e.g., `libmy_contract.so` -> `my_contract`).
fn derive_crate_name(shared_library: &Path) -> eyre::Result<String> {
    let stem = shared_library
        .file_stem()
        .ok_or_else(|| {
            eyre::eyre!(
                "shared library path has no file stem: {}",
                shared_library.display()
            )
        })?
        .to_str()
        .ok_or_else(|| {
            eyre::eyre!(
                "shared library file stem is not valid UTF-8: {}",
                shared_library.display()
            )
        })?;

    let crate_name = stem.strip_prefix("lib").unwrap_or(stem);
    Ok(crate_name.to_string())
}

pub async fn exec(args: Args) -> CargoStylusResult {
    exec_inner(args).await.map_err(Into::into)
}

async fn exec_inner(args: Args) -> eyre::Result<()> {
    let mut contracts = args.project.contracts()?;
    let contract = match contracts.len() {
        1 => contracts.pop().unwrap(),
        _ => bail!("cargo stylus usertrace can only be executed on one contract at a time"),
    };

    let config = BuildConfig {
        features: args.features.unwrap_or_default(),
        ..Default::default()
    };
    // Build the WASM artifact. The native shared library (.so/.dylib) is located separately below.
    let _wasm = contract.build(&config)?;

    let target_dir = Workspace::current()?.metadata.target_directory;
    let library_extension = if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    };
    let shared_library = find_shared_library(target_dir.as_ref(), library_extension)?;
    let crate_name = derive_crate_name(&shared_library)?;

    let provider = args.provider.build_provider().await?;

    // Parent process: spawn rust-stylusdb, which re-invokes this binary with --child.
    // After stylusdb exits, pretty-print the trace file it produced.
    if !args.child {
        let receipt = provider
            .get_transaction_receipt(args.trace.tx)
            .await?
            .ok_or_else(|| eyre::eyre!("no receipt found for tx {}", args.trace.tx))?;
        let to_address = receipt.to.ok_or_else(|| {
            eyre::eyre!(
                "tx {} has no 'to' address (contract creation transactions cannot be traced)",
                args.trace.tx
            )
        })?;
        println!("Tracing contract at address: \x1b[1;32m{to_address:?}\x1b[0m");

        let mut crates_to_trace = vec![crate_name];
        if args.verbose_usertrace {
            crates_to_trace.push("stylus_sdk".to_string());
        }
        crates_to_trace.extend(args.trace_external_usertrace);
        println!("Filtering trace to crates: {}", crates_to_trace.join(", "));
        let pattern = format!("^({})::", crates_to_trace.join("|"));
        let calltrace_cmd = format!("calltrace start '{pattern}'");
        // The trace file path is hardcoded to match what stylusdb writes
        // internally; changing it requires a coordinated update to rust-stylusdb.
        let trace_file = "/tmp/lldb_function_trace.json";
        // Check for symlink before removal to avoid deleting the symlink target.
        match std::fs::symlink_metadata(trace_file) {
            Ok(meta) if !meta.file_type().is_file() => {
                bail!(
                    "stale trace file {trace_file} is not a regular file \
                     (possible symlink attack); refusing to remove"
                );
            }
            Ok(_) => {
                std::fs::remove_file(trace_file)
                    .with_context(|| format!("failed to remove stale trace file {trace_file}"))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => bail!("failed to stat trace file {trace_file}: {e}"),
        }

        if !sys::command_exists("rust-stylusdb") {
            bail!("rust-stylusdb not installed");
        }

        let mut dbg_cmd = Command::new("rust-stylusdb");
        dbg_cmd.args([
            "-o",
            "b user_entrypoint",
            "-o",
            "r",
            "-o",
            &calltrace_cmd,
            "-o",
            "c",
            "-o",
            "calltrace stop",
            "-o",
            "q",
            "--",
        ]);

        dbg_cmd.args(std::env::args());
        dbg_cmd.arg("--child");

        // Silence stylusdb by default for cleaner output. This redirects
        // stdin, stdout, and stderr to null; use --enable-stylusdb-output
        // to preserve them.
        if !args.enable_stylusdb_output {
            dbg_cmd
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        let status = dbg_cmd
            .status()
            .context("failed to execute rust-stylusdb")?;
        if !status.success() {
            bail!(
                "stylusdb returned {}; re-run with --enable-stylusdb-output for diagnostics",
                status
            );
        }

        // Best-effort symlink check: an attacker could place a symlink after
        // remove_file but before stylusdb runs, so this only detects — not
        // prevents — the attack.
        let metadata = std::fs::symlink_metadata(trace_file)
            .with_context(|| format!("failed to stat trace file {trace_file}"))?;
        if !metadata.file_type().is_file() {
            bail!(
                "trace file {trace_file} is not a regular file (possible symlink attack). \
                 WARNING: stylusdb may have already written through the symlink; \
                 inspect the symlink target for unauthorized modifications"
            );
        }

        let status = Command::new("pretty-print-trace")
            .arg(trace_file)
            .status()
            .context("failed to execute pretty-print-trace (is it installed?)")?;
        if !status.success() {
            bail!(
                "pretty-print-trace failed with {}; the raw trace file may still be available at {trace_file}",
                status
            );
        }

        return Ok(());
    }

    // Child process: replay the transaction by calling user_entrypoint in the
    // native shared library, running under stylusdb's call-tracing instrumentation.
    let trace = Trace::new(args.trace.tx, &args.trace.config, &provider).await?;
    let Some(input_args) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = input_args.len();

    unsafe {
        *hostio::FRAME.lock() = Some(trace.reader());

        type Entrypoint = unsafe extern "C" fn(usize) -> usize;
        let lib = libloading::Library::new(&shared_library).with_context(|| {
            format!("failed to load shared library {}", shared_library.display())
        })?;
        let main: libloading::Symbol<Entrypoint> = lib.get(b"user_entrypoint")
            .with_context(|| format!(
                "shared library {} does not export 'user_entrypoint' -- was the contract built correctly?",
                shared_library.display()
            ))?;

        match main(args_len) {
            0 => println!("call completed successfully"),
            1 => println!("call reverted"),
            x => bail!("call exited with unknown status code: {}", x),
        }
    }

    Ok(())
}

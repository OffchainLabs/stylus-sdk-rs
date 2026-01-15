// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! User-level trace command for Stylus contracts
//!
//! This command captures and visualizes user function calls in Stylus contracts
//! using the stylusdb debugger with call tracing enabled.

use alloy::providers::Provider;
use eyre::bail;
use std::{
    path::Path,
    process::{Command, Stdio},
};
use stylus_tools::{
    core::{build::BuildConfig, project::workspace::Workspace, tracing::Trace},
    utils::{color::Color, sys},
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

/// Derive the crate name from a shared library path
fn derive_crate_name(shared_library: &Path) -> String {
    let stem = shared_library
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    let crate_name = stem.strip_prefix("lib").unwrap_or(&stem);
    crate_name.to_string()
}

pub async fn exec(args: Args) -> CargoStylusResult {
    exec_inner(args).await.map_err(Into::into)
}

async fn exec_inner(args: Args) -> eyre::Result<()> {
    let macos = cfg!(target_os = "macos");
    let mut contracts = args.project.contracts()?;
    if contracts.len() != 1 {
        bail!("cargo stylus usertrace can only be executed on one contract at a time");
    }
    let contract = contracts.pop().unwrap();

    // Build the shared library
    let config = BuildConfig {
        features: args.features.unwrap_or_default(),
        ..Default::default()
    };
    let _wasm = contract.build(&config)?;

    let target_dir = Workspace::current()?.metadata.target_directory;
    let library_extension = if macos { ".dylib" } else { ".so" };
    let shared_library = find_shared_library(target_dir.as_ref(), library_extension)?;
    let crate_name = derive_crate_name(&shared_library);

    let provider = args.provider.build_provider().await?;

    // Get the receipt & print the to-address
    if let Some(receipt) = provider.get_transaction_receipt(args.trace.tx).await? {
        if let Some(to_address) = receipt.to {
            println!("Tracing contract at address: \x1b[1;32m{to_address:?}\x1b[0m");
        } else {
            eprintln!("Warning: tx {} has no 'to' address", args.trace.tx);
        }
    } else {
        eprintln!("Warning: no receipt found for tx {}", args.trace.tx);
    }

    // Build the stylusdb calltrace command
    let mut crates_to_trace = vec![crate_name];
    if args.verbose_usertrace {
        crates_to_trace.push("stylus_sdk".to_string());
    }
    crates_to_trace.extend(args.trace_external_usertrace.clone());
    let pattern = format!("^({})::", crates_to_trace.join("|"));
    let calltrace_cmd = format!("calltrace start '{pattern}'");

    // Non-child: spawn stylusdb + pretty-print
    if !args.child {
        // Remove any stale LLDB trace
        let _ = std::fs::remove_file("/tmp/lldb_function_trace.json");

        // Invoke stylusdb
        let (cmd_name, cmd_args) = if sys::command_exists("rust-stylusdb") {
            (
                "rust-stylusdb",
                &[
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
                ][..],
            )
        } else {
            bail!("rust-stylusdb not installed");
        };

        let mut dbg_cmd = Command::new(cmd_name);
        dbg_cmd.args(cmd_args);

        // Forward all original args and append child flag
        for a in std::env::args() {
            dbg_cmd.arg(a);
        }
        dbg_cmd.arg("--child");

        if !args.enable_stylusdb_output {
            dbg_cmd
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        let status = dbg_cmd.status()?;
        if !status.success() {
            bail!("stylusdb returned {}", status);
        }

        // Now pretty-print the trace
        let mut pp = Command::new("pretty-print-trace");
        pp.arg("/tmp/lldb_function_trace.json");

        let mut child = pp.spawn()?;
        let _ = child.wait();

        return Ok(());
    }

    // Replay the WASM (child process)
    let trace = Trace::new(args.trace.tx, &args.trace.config, &provider).await?;
    let Some(input_args) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = input_args.len();

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

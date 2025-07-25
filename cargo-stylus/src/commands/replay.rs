// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::iter;

use eyre::bail;
use stylus_tools::{
    core::{
        build::build_shared_library,
        debugger::Debugger,
        tracing::{hostios, Trace},
    },
    utils::color::Color,
};

use crate::{
    common_args::{ProviderArgs, TraceArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Which specific package to build during replay, if any.
    #[arg(long)]
    package: Option<String>,
    /// Whether this process is the child of another.
    ///
    /// The parent process launches the debugger, with the same arguments plus this `--child` flag.
    /// The child process then runs the transaction.
    #[arg(long, hide(true))]
    child: bool,

    #[command(flatten)]
    trace: TraceArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    if args.child {
        exec_replay_transaction(args).await?;
    } else {
        launch_debugger()?;
    }
    Ok(())
}

async fn exec_replay_transaction(args: Args) -> eyre::Result<()> {
    let provider = args.provider.build_provider().await?;
    let trace = Trace::new(args.trace.tx, args.trace.use_native_tracer, &provider).await?;
    let shared_library = build_shared_library(&args.trace.project, args.package, None)?;

    // TODO: don't assume the contract is top-level
    let Some(args) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = args.len();

    unsafe {
        *hostios::FRAME.lock() = Some(trace.into_frame_reader());

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

// TODO: Use "never" type when feature is stabilized rust-lang/rust#35121
fn launch_debugger() -> eyre::Result<()> {
    let debugger = Debugger::select()?;
    let args = std::env::args().chain(iter::once("--child".to_string()));
    let err = debugger.exec(args);
    eyre::bail!("failed to exec {}: {}", debugger.program, err);
}

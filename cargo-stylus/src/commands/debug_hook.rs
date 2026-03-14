// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Debugger hook infrastructure for Stylus contract debugging.
//!
//! This module provides hooks for debugger integration. The trait methods
//! `on_external_call` and `on_return_from_call` are scaffolded for future
//! Solidity interop debugging support. The remaining methods are called
//! directly on the `StylusDebuggerHook` instance during debugger setup in
//! `replay.rs`.

use eyre::Result;
use parking_lot::Mutex;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Interface for debugger hooks to receive execution events.
#[allow(dead_code)]
pub(crate) trait DebuggerHook: Send + Sync {
    /// Called when execution enters an external contract
    fn on_external_call(&self, contract_address: &str);

    /// Called when returning from an external contract call
    fn on_return_from_call(&self);

    /// Called at the start of execution with the set of contract address-to-source-path
    /// mappings. Each tuple is `(address, source_path)`.
    fn on_execution_start(&self, contracts: &[(String, String)]);

    /// Called to register whether a contract is Solidity or Stylus.
    fn on_contract_info(&self, contract_address: &str, is_solidity: bool);
}

/// No-op implementation for when no debugger is attached
#[allow(dead_code)]
pub(crate) struct NoOpDebuggerHook;

impl DebuggerHook for NoOpDebuggerHook {
    fn on_external_call(&self, _contract_address: &str) {}
    fn on_return_from_call(&self) {}
    fn on_execution_start(&self, _contracts: &[(String, String)]) {}
    fn on_contract_info(&self, _contract_address: &str, _is_solidity: bool) {}
}

/// Stylus debugger hook that communicates via Unix socket.
///
/// TODO: Add cross-platform support. Currently this module requires Unix
/// (it imports `std::os::unix::net`). Windows support would require either
/// Windows Subsystem for Linux (WSL) or a platform-specific implementation
/// using named pipes.
///
/// TODO: The `JoinHandle` from the listener thread is discarded. If no debugger
/// connects, the thread blocks on `accept()` indefinitely and is leaked until
/// process exit. Dropping `StylusDebuggerHook` removes the socket file but does
/// not unblock the listener thread. Consider storing the `JoinHandle` and
/// implementing graceful shutdown (e.g., via a shutdown flag and a non-blocking
/// accept loop).
pub(crate) struct StylusDebuggerHook {
    socket_path: String,
    connection: Arc<Mutex<Option<UnixStream>>>,
    warned_no_connection: Arc<AtomicBool>,
}

impl StylusDebuggerHook {
    pub(crate) fn new() -> Result<Self> {
        let socket_path = format!("/tmp/stylus_debug_{}.sock", std::process::id());
        let connection = Arc::new(Mutex::new(None));
        let warned_no_connection = Arc::new(AtomicBool::new(false));

        // Bind on the main thread so errors propagate directly to the caller,
        // avoiding a channel just to relay bind status from the listener thread.
        let listener = Self::bind_listener(&socket_path)?;

        let listener_path = socket_path.clone();
        let listener_conn = Arc::clone(&connection);
        let listener_warned = Arc::clone(&warned_no_connection);
        std::thread::Builder::new()
            .name("stylus-debug-listener".into())
            .spawn(move || {
                Self::accept_connection(&listener_path, listener, &listener_conn, &listener_warned)
            })
            .map_err(|e| eyre::eyre!("failed to spawn debugger listener thread: {e}"))?;

        Ok(Self {
            socket_path,
            connection,
            warned_no_connection,
        })
    }

    /// Remove a socket file, tolerating `NotFound`.
    fn remove_socket_file(path: &str) -> std::io::Result<()> {
        match std::fs::remove_file(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }

    /// Bind a Unix listener, attempting a direct bind first. If `AddrInUse` is
    /// returned, removes the stale socket file and retries. This avoids the
    /// TOCTOU race of check-then-remove-then-bind.
    fn bind_listener(socket_path: &str) -> Result<UnixListener> {
        let first_err = match UnixListener::bind(socket_path) {
            Ok(listener) => return Ok(listener),
            Err(e) => e,
        };
        if first_err.kind() != std::io::ErrorKind::AddrInUse {
            return Err(first_err.into());
        }
        Self::remove_socket_file(socket_path).map_err(|e| {
            eyre::eyre!("failed to remove existing debug socket {socket_path}: {e}")
        })?;
        UnixListener::bind(socket_path).map_err(|e| {
            eyre::eyre!(
                "failed to rebind debug socket {socket_path} after removing existing file: {e}"
            )
        })
    }

    /// Wait for a debugger to connect. Called on the listener thread after bind
    /// has already succeeded on the main thread.
    fn accept_connection(
        socket_path: &str,
        listener: UnixListener,
        connection: &Mutex<Option<UnixStream>>,
        warned_no_connection: &AtomicBool,
    ) {
        match listener.accept() {
            Ok((stream, _)) => {
                eprintln!("Debugger connected via socket");
                *connection.lock() = Some(stream);
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to accept debugger connection on {socket_path}: {e}. \
                     Debugger commands will not be delivered this session."
                );
                // Suppress the redundant "no active connection" warning in send_command.
                warned_no_connection.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Sanitize a value for inclusion in a debugger command.
    /// Rejects values containing newlines or carriage returns which could inject
    /// additional commands into the line-oriented debugger protocol.
    fn sanitize_field(value: &str) -> Option<&str> {
        if value.contains('\n') || value.contains('\r') {
            eprintln!(
                "Warning: debugger command field contains newline; \
                 dropping to prevent command injection"
            );
            None
        } else {
            Some(value)
        }
    }

    /// Send a command to the debugger. Fire-and-forget: errors are reported via
    /// stderr only and do not propagate to callers. After the first write failure,
    /// the connection is closed and subsequent calls are silently dropped.
    fn send_command(&self, command: &str) {
        let mut conn_guard = self.connection.lock();
        if let Some(stream) = conn_guard.as_mut() {
            if let Err(e) = writeln!(stream, "{command}") {
                eprintln!(
                    "Warning: failed to send debugger command: {e}; \
                     future commands will be dropped"
                );
                // Drop the broken stream so subsequent calls hit the
                // "no active connection" path instead of repeated write errors.
                *conn_guard = None;
                // Relaxed is sufficient: this flag only gates a warning message
                // and all data access is mutex-protected.
                self.warned_no_connection.store(true, Ordering::Relaxed);
            }
        } else if !self.warned_no_connection.swap(true, Ordering::Relaxed) {
            eprintln!("Warning: debugger commands dropped: no active connection");
        }
    }
}

impl DebuggerHook for StylusDebuggerHook {
    fn on_external_call(&self, contract_address: &str) {
        if let Some(addr) = Self::sanitize_field(contract_address) {
            self.send_command(&format!("switch_context {addr}"));
        }
    }

    fn on_return_from_call(&self) {
        self.send_command("return_from_call");
    }

    fn on_execution_start(&self, contracts: &[(String, String)]) {
        for (address, path) in contracts {
            if let (Some(addr), Some(p)) =
                (Self::sanitize_field(address), Self::sanitize_field(path))
            {
                self.send_command(&format!("contract_add {addr} {p}"));
            }
        }
    }

    fn on_contract_info(&self, contract_address: &str, is_solidity: bool) {
        if let Some(addr) = Self::sanitize_field(contract_address) {
            let contract_type = if is_solidity { "solidity" } else { "stylus" };
            self.send_command(&format!("contract_type {addr} {contract_type}"));
        }
    }
}

impl Drop for StylusDebuggerHook {
    fn drop(&mut self) {
        if let Err(e) = Self::remove_socket_file(&self.socket_path) {
            eprintln!(
                "Warning: failed to clean up debug socket {}: {e}",
                self.socket_path
            );
        }
    }
}

/// Global hook so that callbacks (e.g., future hostio integrations) can access
/// the debugger without threading it through every call.
pub(crate) static DEBUGGER_HOOK: Mutex<Option<Arc<dyn DebuggerHook>>> = Mutex::new(None);

/// Set the global debugger hook. Expected to be called exactly once during
/// debugger setup; calling it again replaces the existing hook and emits a warning.
pub(crate) fn init_debugger_hook(hook: Arc<dyn DebuggerHook>) {
    let mut guard = DEBUGGER_HOOK.lock();
    if guard.is_some() {
        eprintln!("Warning: replacing existing debugger hook");
    }
    *guard = Some(hook);
}

/// Get the current debugger hook.
/// Not yet called; will be used when Solidity interop debugging is added.
#[allow(dead_code)]
pub(crate) fn get_debugger_hook() -> Option<Arc<dyn DebuggerHook>> {
    DEBUGGER_HOOK.lock().clone()
}

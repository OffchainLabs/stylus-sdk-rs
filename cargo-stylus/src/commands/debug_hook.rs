// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Debugger hook infrastructure for Stylus contract debugging.
//!
//! This module provides hooks for debugger integration, including support for
//! cross-contract context switching. Some methods are scaffolded for future
//! Solidity interop debugging support.

use eyre::Result;
use parking_lot::Mutex;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;

/// Interface for debugger hooks to receive execution events.
///
/// Note: `on_external_call` and `on_return_from_call` are reserved for future
/// Stylus <-> Solidity interop debugging support.
#[allow(dead_code)]
pub trait DebuggerHook: Send + Sync {
    /// Called when execution enters an external contract
    fn on_external_call(&self, contract_address: &str);

    /// Called when returning from an external contract call
    fn on_return_from_call(&self);

    /// Called when execution starts
    fn on_execution_start(&self, contracts: &[(String, String)]);

    /// Called to register contract metadata
    fn on_contract_info(&self, contract_address: &str, is_solidity: bool);
}

/// No-op implementation for when no debugger is attached
#[allow(dead_code)]
pub struct NoOpDebuggerHook;

impl DebuggerHook for NoOpDebuggerHook {
    fn on_external_call(&self, _contract_address: &str) {}
    fn on_return_from_call(&self) {}
    fn on_execution_start(&self, _contracts: &[(String, String)]) {}
    fn on_contract_info(&self, _contract_address: &str, _is_solidity: bool) {}
}

/// Stylus debugger hook that communicates via Unix socket.
///
/// TODO: Windows is not currently a target for stylusdb-based debugging.
/// This implementation uses Unix sockets and Unix-specific paths (/tmp/).
/// Windows support would require either Windows Subsystem for Linux (WSL)
/// or a platform-specific implementation using named pipes.
pub struct StylusDebuggerHook {
    socket_path: String,
    connection: Arc<Mutex<Option<UnixStream>>>,
}

impl StylusDebuggerHook {
    pub fn new() -> Result<Self> {
        // Note: Unix-specific path format; not supported on Windows without WSL
        let socket_path = format!("/tmp/stylus_debug_{}.sock", std::process::id());

        // Start listener in background
        let path_clone = socket_path.clone();
        std::thread::spawn(move || {
            if let Err(err) = Self::listen_for_connection(&path_clone) {
                eprintln!("Failed to establish debugger connection: {err}");
            }
        });

        Ok(Self {
            socket_path,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    fn listen_for_connection(socket_path: &str) -> Result<()> {
        // Remove existing socket if it exists
        if Path::new(socket_path).exists() {
            std::fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;
        listener.set_nonblocking(false)?;

        // Wait for debugger to connect
        if let Ok((_stream, _)) = listener.accept() {
            println!("Debugger connected via socket");
            // Store connection for later use
            // TODO: Pass the stream back to store in self.connection.
            // This will be needed when Solidity interop debugging is added,
            // enabling real-time context switching between Stylus and Solidity contracts.
        }

        Ok(())
    }

    fn send_command(&self, command: &str) {
        let mut conn_guard = self.connection.lock();
        if let Some(ref mut stream) = *conn_guard {
            let _ = writeln!(stream, "{command}");
        }
    }
}

impl DebuggerHook for StylusDebuggerHook {
    fn on_external_call(&self, contract_address: &str) {
        self.send_command(&format!("switch_context {contract_address}"));
    }

    fn on_return_from_call(&self) {
        self.send_command("return_from_call");
    }

    fn on_execution_start(&self, contracts: &[(String, String)]) {
        // Send all contract mappings to debugger
        for (address, path) in contracts {
            self.send_command(&format!("contract_add {address} {path}"));
        }
    }

    fn on_contract_info(&self, contract_address: &str, is_solidity: bool) {
        let contract_type = if is_solidity { "solidity" } else { "stylus" };
        self.send_command(&format!("contract_type {contract_address} {contract_type}"));
    }
}

impl Drop for StylusDebuggerHook {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Global debugger hook instance
pub static DEBUGGER_HOOK: Mutex<Option<Arc<dyn DebuggerHook>>> = Mutex::new(None);

/// Initialize the debugger hook
pub fn init_debugger_hook(hook: Arc<dyn DebuggerHook>) {
    let mut guard = DEBUGGER_HOOK.lock();
    *guard = Some(hook);
}

/// Get the current debugger hook.
/// Reserved for future Solidity interop debugging support.
#[allow(dead_code)]
pub fn get_debugger_hook() -> Option<Arc<dyn DebuggerHook>> {
    DEBUGGER_HOOK.lock().clone()
}

// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! User-level trace command for Stylus contracts
//!
//! This command captures and visualizes user function calls in Stylus contracts
//! using the stylusdb debugger with call tracing enabled.

use alloy::providers::Provider;
use eyre::bail;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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
    utils::{hostio, soldb_bridge},
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

    /// URL of the cross-environment debug bridge server for Solidity interop.
    /// Example: --cross-env-bridge http://127.0.0.1:8765
    #[arg(long)]
    cross_env_bridge: Option<String>,

    /// Path to JSON file containing Solidity contract configurations.
    /// Format: [{ "address": "0x...", "name": "ContractName", "debug_dir": "./path" }]
    #[arg(long)]
    solidity_contracts: Option<PathBuf>,
}

/// Contract configuration from JSON
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContractConfig {
    pub address: String,
    pub environment: String,
    pub name: String,
    pub project_path: String,
    pub debug_dir: String,
}

/// Config file format: { "contracts": [...] }
#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigFile {
    contracts: Vec<ContractConfig>,
}

/// Parse contracts from JSON config file
/// Format: { "contracts": [{ "address": "0x...", "environment": "evm", "name": "...", "project_path": "...", "debug_dir": "..." }] }
fn parse_solidity_contracts(path: &Path) -> eyre::Result<HashMap<String, ContractConfig>> {
    let content = std::fs::read_to_string(path)?;
    let config: ConfigFile = serde_json::from_str(&content)?;

    let mut contracts = HashMap::new();

    for contract in config.contracts {
        // Only include EVM contracts for Solidity interop
        if contract.environment == "evm" {
            let normalized_addr = contract.address.to_lowercase();
            contracts.insert(normalized_addr, contract);
        }
    }

    Ok(contracts)
}

/// Initialize cross-environment bridge with Solidity contracts
fn init_cross_env_bridge(
    bridge_url: &str,
    solidity_contracts: &HashMap<String, ContractConfig>,
    tx_hash: Option<&str>,
    caller_address: Option<&str>,
    block_number: Option<u64>,
) -> eyre::Result<()> {
    soldb_bridge::set_bridge_url(bridge_url);

    let mut client = soldb_bridge::SoldbBridgeClient::new(Some(bridge_url));
    if !client.connect()? {
        bail!("Failed to connect to cross-env bridge at {}", bridge_url);
    }

    // Register all Solidity contracts
    for (addr, config) in solidity_contracts {
        client.register_evm_contract(
            addr,
            &config.name,
            Some(&config.debug_dir),
            &config.project_path,
        )?;
    }

    hostio::set_cross_env_config(soldb_bridge::CrossEnvConfig {
        bridge_url: bridge_url.to_string(),
        solidity_contracts: solidity_contracts.keys().cloned().collect(),
        tx_hash: tx_hash.map(String::from),
        caller_address: caller_address.map(String::from),
        block_number,
    });

    Ok(())
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

/// Merge cross-env (Solidity) traces into the StylusDB function trace JSON.
///
/// This reads the cross_env_path file (written by hostio.rs child process)
/// and merges the Solidity calls as children of the matching Stylus calls.
fn merge_cross_env_traces(lldb_trace_path: &str, cross_env_path: &str) -> eyre::Result<bool> {
    if !std::path::Path::new(cross_env_path).exists() {
        return Ok(false);
    }

    let lldb_json = std::fs::read_to_string(lldb_trace_path)?;
    let cross_env_json = std::fs::read_to_string(cross_env_path)?;

    let lldb_parsed: serde_json::Value = serde_json::from_str(&lldb_json)?;
    let mut lldb_trace = lldb_parsed
        .get("calls")
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| {
            eyre::eyre!("invalid LLDB trace format: expected object with 'calls' field")
        })?;

    let cross_env_traces: Vec<serde_json::Value> = serde_json::from_str(&cross_env_json)?;

    if cross_env_traces.is_empty() {
        let _ = std::fs::remove_file(cross_env_path);
        return Ok(false);
    }

    // Track if any cross-env trace failed (for final status and error header)
    let mut any_trace_failed = false;
    let mut first_error_message: Option<String> = None;

    // Find max call_id in LLDB trace
    let mut max_call_id: u64 = lldb_trace
        .iter()
        .filter_map(|e| e.get("call_id").and_then(|v| v.as_u64()))
        .max()
        .unwrap_or(0);

    for cross_env_entry in &cross_env_traces {
        let target_address = cross_env_entry
            .get("target_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        let trace = match cross_env_entry.get("trace") {
            Some(t) => t,
            None => continue,
        };

        let evm_calls = trace
            .get("calls")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if evm_calls.is_empty() {
            continue;
        }

        // Find the parent LLDB call for this cross-env trace.
        // Strategy: find the call where an arg value contains the target address
        // (this is the interface `::new` call), then find its next sibling
        // (the actual method call like `::add`).
        let mut parent_call_id: Option<u64> = None;

        // Step 1: Find the ::new call that has the address in args
        let mut new_call_parent: Option<u64> = None;
        let mut new_call_id: Option<u64> = None;
        for entry in &lldb_trace {
            if let Some(args) = entry.get("args").and_then(|a| a.as_array()) {
                for arg in args {
                    if let Some(val) = arg.get("value").and_then(|v| v.as_str()) {
                        if val.to_lowercase().contains(&target_address) {
                            new_call_id = entry.get("call_id").and_then(|v| v.as_u64());
                            new_call_parent = entry.get("parent_call_id").and_then(|v| v.as_u64());
                            break;
                        }
                    }
                }
            }
            if new_call_id.is_some() {
                break;
            }
        }

        // Step 2: Find the next sibling (same parent, higher call_id)
        if let (Some(new_cid), Some(new_parent)) = (new_call_id, new_call_parent) {
            let mut best_sibling: Option<u64> = None;
            for entry in &lldb_trace {
                let cid = entry.get("call_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let pid = entry
                    .get("parent_call_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if pid == new_parent && cid > new_cid {
                    if best_sibling.map_or(true, |b| cid < b) {
                        best_sibling = Some(cid);
                    }
                }
            }
            parent_call_id = best_sibling;
        }

        // Fallback: use the last call in the trace
        if parent_call_id.is_none() {
            parent_call_id = lldb_trace
                .last()
                .and_then(|e| e.get("call_id").and_then(|v| v.as_u64()));
        }

        let stylus_parent_id = parent_call_id.unwrap_or(0);

        // Build mapping from original EVM call_ids to new call_ids
        // so we preserve the EVM hierarchy
        let mut id_map: HashMap<u64, u64> = HashMap::new();
        for evm_call in &evm_calls {
            let orig_id = evm_call
                .get("call_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            max_call_id += 1;
            id_map.insert(orig_id, max_call_id);
        }

        // Check trace-level error (applies to all calls in this trace)
        let trace_success = trace
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Track if this trace failed
        if !trace_success {
            any_trace_failed = true;
            // Try to find the first error message from calls
            if first_error_message.is_none() {
                for evm_call in &evm_calls {
                    if let Some(err) = evm_call.get("error").and_then(|v| v.as_str()) {
                        if !err.is_empty() {
                            first_error_message = Some(err.to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Insert EVM calls with correct hierarchy
        for evm_call in &evm_calls {
            let orig_id = evm_call
                .get("call_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let orig_parent = evm_call.get("parent_call_id").and_then(|v| v.as_u64());

            let new_id = id_map[&orig_id];

            // If this call has a parent in the EVM trace, map it.
            // Otherwise it's a root EVM call → parent is the Stylus call.
            let new_parent = match orig_parent {
                Some(pid) if id_map.contains_key(&pid) => id_map[&pid],
                _ => stylus_parent_id,
            };

            let func_name = evm_call
                .get("function_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Extract source location if available
            let (file, line) = if let Some(src_loc) = evm_call.get("source_location") {
                let f = src_loc.get("file").and_then(|v| v.as_str()).unwrap_or("");
                let l = src_loc.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                (f.to_string(), l)
            } else {
                (String::new(), 0)
            };

            // Build args array from EVM call args
            let mut args_array = Vec::new();
            if let Some(args) = evm_call.get("args").and_then(|a| a.as_array()) {
                for arg in args {
                    let name = arg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let arg_type = arg.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let value = arg.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    args_array.push(serde_json::json!({
                        "name": name,
                        "value": format!("{}: {}", arg_type, value),
                    }));
                }
            }

            // Check if this EVM call failed - check multiple possible fields
            let call_success = evm_call
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            // error can be a string (error message) or a boolean
            let has_error_field = evm_call
                .get("error")
                .map(|v| {
                    // Check if error is a non-empty string or true boolean
                    v.as_str().map(|s| !s.is_empty()).unwrap_or(false)
                        || v.as_bool().unwrap_or(false)
                })
                .unwrap_or(false);

            // Call failed if: success=false OR error field is set
            let call_failed = !call_success || has_error_field;

            // Get error message from call or trace level
            let error_msg = evm_call
                .get("error")
                .and_then(|v| v.as_str())
                .map(String::from)
                .or_else(|| {
                    evm_call
                        .get("error_message")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                });

            let mut new_entry = serde_json::json!({
                "call_id": new_id,
                "parent_call_id": new_parent,
                "function": format!("[EVM] {}", func_name),
                "file": file,
                "line": line,
                "args": args_array,
                "environment": "evm",
                "contract_address": evm_call.get("contract_address").and_then(|v| v.as_str()).unwrap_or(""),
                "gas_used": evm_call.get("gas_used"),
                "return_data": evm_call.get("return_data"),
            });

            // Add error fields if the call failed
            if call_failed {
                new_entry["error"] = serde_json::json!(true);
                if let Some(ref msg) = error_msg {
                    new_entry["error_message"] = serde_json::json!(msg);
                }
            }

            lldb_trace.push(new_entry);
        }
    }

    // Determine final status based on cross-env trace results
    let final_status = if any_trace_failed { "error" } else { "success" };

    // Write back in new format with status, calls, and error_message for header
    let mut output = serde_json::json!({
        "status": final_status,
        "calls": lldb_trace
    });

    // Add error_message at top level for header display
    if let Some(ref msg) = first_error_message {
        output["error_message"] = serde_json::json!(msg);
    }

    let merged_json = serde_json::to_string_pretty(&output)?;
    std::fs::write(lldb_trace_path, merged_json)?;

    let _ = std::fs::remove_file(cross_env_path);

    Ok(true)
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

    let solidity_contracts = if let Some(ref path) = args.solidity_contracts {
        parse_solidity_contracts(path)?
    } else {
        HashMap::new()
    };

    let (caller_address, block_number) = if let Some(tx) = args.trace.tx {
        let tx_info = provider.get_transaction_by_hash(tx).await?;
        if let Some(tx_data) = tx_info {
            let caller = format!("{:?}", tx_data.inner.signer());
            let block = tx_data.block_number;
            (Some(caller), block)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    if let Some(ref bridge_url) = args.cross_env_bridge {
        let tx_hash_str = args.trace.tx.map(|h| format!("{:?}", h));
        init_cross_env_bridge(
            bridge_url,
            &solidity_contracts,
            tx_hash_str.as_deref(),
            caller_address.as_deref(),
            block_number,
        )?;
    }

    let _trace = if args.trace.simulate {
        Trace::simulate(&args.trace.simulation, &args.trace.config, &provider).await?
    } else {
        let tx = args
            .trace
            .tx
            .ok_or_else(|| eyre::eyre!("missing transaction hash and not in simulate mode"))?;
        Trace::new(tx, &args.trace.config, &provider).await?
    };

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
        // Remove any stale trace files
        let _ = std::fs::remove_file("/tmp/lldb_function_trace.json");
        let _ = std::fs::remove_file(hostio::CROSS_ENV_TRACES_PATH);

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

        // Merge cross-env (Solidity) traces into LLDB trace if any exist
        if let Err(e) = merge_cross_env_traces(
            "/tmp/lldb_function_trace.json",
            hostio::CROSS_ENV_TRACES_PATH,
        ) {
            bail!("failed to merge cross-env traces: {}", e);
        }

        // Now pretty-print the trace
        let mut pp = Command::new("pretty-print-trace");
        pp.arg("/tmp/lldb_function_trace.json");

        let mut child = pp.spawn()?;
        let _ = child.wait();

        return Ok(());
    }

    // Replay the WASM (child process)
    let trace = if args.trace.simulate {
        Trace::simulate(&args.trace.simulation, &args.trace.config, &provider).await?
    } else {
        let tx = args
            .trace
            .tx
            .ok_or_else(|| eyre::eyre!("missing transaction hash in child process"))?;
        Trace::new(tx, &args.trace.config, &provider).await?
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temporary JSON config file
    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_solidity_contracts_single_evm_contract() {
        let config = r#"{
            "contracts": [
                {
                    "address": "0x1234567890abcdef1234567890abcdef12345678",
                    "environment": "evm",
                    "name": "TestToken",
                    "project_path": "/path/to/project",
                    "debug_dir": "/path/to/debug"
                }
            ]
        }"#;

        let file = create_temp_config(config);
        let result = parse_solidity_contracts(file.path()).unwrap();

        assert_eq!(result.len(), 1);
        let contract = result
            .get("0x1234567890abcdef1234567890abcdef12345678")
            .unwrap();
        assert_eq!(contract.name, "TestToken");
        assert_eq!(contract.environment, "evm");
        assert_eq!(contract.project_path, "/path/to/project");
        assert_eq!(contract.debug_dir, "/path/to/debug");
    }

    #[test]
    fn test_parse_solidity_contracts_multiple_contracts() {
        let config = r#"{
            "contracts": [
                {
                    "address": "0xaaaa567890abcdef1234567890abcdef12345678",
                    "environment": "evm",
                    "name": "TokenA",
                    "project_path": "/path/a",
                    "debug_dir": "/debug/a"
                },
                {
                    "address": "0xbbbb567890abcdef1234567890abcdef12345678",
                    "environment": "evm",
                    "name": "TokenB",
                    "project_path": "/path/b",
                    "debug_dir": "/debug/b"
                },
                {
                    "address": "0xcccc567890abcdef1234567890abcdef12345678",
                    "environment": "stylus",
                    "name": "StylusContract",
                    "project_path": "/path/c",
                    "debug_dir": "/debug/c"
                }
            ]
        }"#;

        let file = create_temp_config(config);
        let result = parse_solidity_contracts(file.path()).unwrap();

        // Only EVM contracts should be included
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("0xaaaa567890abcdef1234567890abcdef12345678"));
        assert!(result.contains_key("0xbbbb567890abcdef1234567890abcdef12345678"));
        // Stylus contract should be excluded
        assert!(!result.contains_key("0xcccc567890abcdef1234567890abcdef12345678"));
    }

    #[test]
    fn test_parse_solidity_contracts_missing_file() {
        let result = parse_solidity_contracts(Path::new("/nonexistent/path/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_cross_env_traces_no_cross_env_file() {
        // Create a temporary LLDB trace file (format with status and calls)
        let lldb_trace = r#"{
            "status": "success",
            "calls": [
                {"call_id": 1, "function": "main", "file": "lib.rs", "line": 10, "args": []}
            ]
        }"#;

        let lldb_file = NamedTempFile::new().unwrap();
        std::fs::write(lldb_file.path(), lldb_trace).unwrap();

        // Use a non-existent temp path for cross-env file
        let cross_env_file = NamedTempFile::new().unwrap();
        let cross_env_path = cross_env_file.path().to_str().unwrap().to_string();
        drop(cross_env_file); // Delete the file so it doesn't exist

        let result = merge_cross_env_traces(lldb_file.path().to_str().unwrap(), &cross_env_path);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false when no cross-env file
    }

    #[test]
    fn test_merge_cross_env_traces_empty_cross_env() {
        let lldb_trace = r#"{
            "status": "success",
            "calls": [
                {"call_id": 1, "function": "main", "file": "lib.rs", "line": 10, "args": []}
            ]
        }"#;

        let lldb_file = NamedTempFile::new().unwrap();
        std::fs::write(lldb_file.path(), lldb_trace).unwrap();

        // Create empty cross-env traces file
        let cross_env_file = NamedTempFile::new().unwrap();
        std::fs::write(cross_env_file.path(), "[]").unwrap();

        let result = merge_cross_env_traces(
            lldb_file.path().to_str().unwrap(),
            cross_env_file.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false when empty
    }

    #[test]
    fn test_merge_cross_env_traces_with_evm_calls() {
        let lldb_trace = r#"{
            "status": "success",
            "calls": [
                {"call_id": 1, "parent_call_id": 0, "function": "user_entrypoint", "file": "lib.rs", "line": 1, "args": []},
                {"call_id": 2, "parent_call_id": 1, "function": "IToken::new", "file": "lib.rs", "line": 10, "args": [{"name": "addr", "value": "0xabcd1234"}]},
                {"call_id": 3, "parent_call_id": 1, "function": "IToken::transfer", "file": "lib.rs", "line": 15, "args": []}
            ]
        }"#;

        let cross_env_trace = r#"[
            {
                "target_address": "0xabcd1234",
                "calldata": "0xa9059cbb",
                "call_type": "CALL",
                "trace": {
                    "trace_id": "test-123",
                    "calls": [
                        {
                            "call_id": 1,
                            "function_name": "transfer",
                            "contract_address": "0xabcd1234",
                            "environment": "evm",
                            "call_type": "external",
                            "success": true,
                            "args": [
                                {"name": "to", "type": "address", "value": "0x9999"},
                                {"name": "amount", "type": "uint256", "value": "1000"}
                            ]
                        }
                    ]
                }
            }
        ]"#;

        let lldb_file = NamedTempFile::new().unwrap();
        std::fs::write(lldb_file.path(), lldb_trace).unwrap();
        let cross_env_file = NamedTempFile::new().unwrap();
        std::fs::write(cross_env_file.path(), cross_env_trace).unwrap();

        let result = merge_cross_env_traces(
            lldb_file.path().to_str().unwrap(),
            cross_env_file.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true when merged

        // Verify merged content
        let merged = std::fs::read_to_string(lldb_file.path()).unwrap();
        let merged_parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        let merged_json = merged_parsed.get("calls").unwrap().as_array().unwrap();

        // Should have original 3 calls + 1 EVM call
        assert_eq!(merged_json.len(), 4);

        // Find the EVM call
        let evm_call = merged_json
            .iter()
            .find(|c| {
                c.get("function")
                    .and_then(|f| f.as_str())
                    .map(|f| f.starts_with("[EVM]"))
                    .unwrap_or(false)
            })
            .unwrap();

        assert_eq!(
            evm_call.get("function").unwrap().as_str().unwrap(),
            "[EVM] transfer"
        );
        assert_eq!(
            evm_call.get("environment").unwrap().as_str().unwrap(),
            "evm"
        );
    }

    #[test]
    fn test_merge_cross_env_traces_preserves_evm_hierarchy() {
        let lldb_trace = r#"{
            "status": "success",
            "calls": [
                {"call_id": 1, "parent_call_id": 0, "function": "main", "file": "lib.rs", "line": 1, "args": []}
            ]
        }"#;

        // EVM trace with nested calls
        let cross_env_trace = r#"[
            {
                "target_address": "0xtest1234",
                "calldata": "0x",
                "call_type": "CALL",
                "trace": {
                    "trace_id": "hierarchy-test",
                    "calls": [
                        {
                            "call_id": 1,
                            "function_name": "outerCall",
                            "contract_address": "0xtest1234",
                            "environment": "evm",
                            "call_type": "external",
                            "success": true,
                            "args": []
                        },
                        {
                            "call_id": 2,
                            "parent_call_id": 1,
                            "function_name": "innerCall",
                            "contract_address": "0xtest5678",
                            "environment": "evm",
                            "call_type": "external",
                            "success": true,
                            "args": []
                        }
                    ]
                }
            }
        ]"#;

        let lldb_file = NamedTempFile::new().unwrap();
        std::fs::write(lldb_file.path(), lldb_trace).unwrap();
        let cross_env_file = NamedTempFile::new().unwrap();
        std::fs::write(cross_env_file.path(), cross_env_trace).unwrap();

        let result = merge_cross_env_traces(
            lldb_file.path().to_str().unwrap(),
            cross_env_file.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());

        let merged = std::fs::read_to_string(lldb_file.path()).unwrap();
        let merged_parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        let merged_json = merged_parsed.get("calls").unwrap().as_array().unwrap();

        // Should have 1 original + 2 EVM calls
        assert_eq!(merged_json.len(), 3);

        // Find outer and inner EVM calls
        let outer = merged_json
            .iter()
            .find(|c| {
                c.get("function")
                    .and_then(|f| f.as_str())
                    .map(|f| f.contains("outerCall"))
                    .unwrap_or(false)
            })
            .unwrap();

        let inner = merged_json
            .iter()
            .find(|c| {
                c.get("function")
                    .and_then(|f| f.as_str())
                    .map(|f| f.contains("innerCall"))
                    .unwrap_or(false)
            })
            .unwrap();

        let outer_id = outer.get("call_id").unwrap().as_u64().unwrap();
        let inner_parent = inner.get("parent_call_id").unwrap().as_u64().unwrap();

        // Inner call's parent should be the outer call
        assert_eq!(inner_parent, outer_id);
    }

    #[test]
    fn test_merge_cross_env_traces_propagates_evm_errors() {
        let lldb_trace = r#"{
            "status": "success",
            "calls": [
                {"call_id": 1, "parent_call_id": 0, "function": "user_entrypoint", "file": "lib.rs", "line": 1, "args": []},
                {"call_id": 2, "parent_call_id": 1, "function": "IToken::new", "file": "lib.rs", "line": 10, "args": [{"name": "addr", "value": "0xfailed1234"}]},
                {"call_id": 3, "parent_call_id": 1, "function": "IToken::multiply", "file": "lib.rs", "line": 15, "args": []}
            ]
        }"#;

        // EVM trace with a failed call
        let cross_env_trace = r#"[
            {
                "target_address": "0xfailed1234",
                "calldata": "0xa9059cbb",
                "call_type": "CALL",
                "trace": {
                    "trace_id": "failed-trace",
                    "success": false,
                    "calls": [
                        {
                            "call_id": 1,
                            "function_name": "multiply",
                            "contract_address": "0xfailed1234",
                            "environment": "evm",
                            "call_type": "external",
                            "success": false,
                            "error": "Arithmetic overflow",
                            "args": [
                                {"name": "a", "type": "uint256", "value": "10"},
                                {"name": "b", "type": "uint256", "value": "0"}
                            ]
                        }
                    ]
                }
            }
        ]"#;

        let lldb_file = NamedTempFile::new().unwrap();
        std::fs::write(lldb_file.path(), lldb_trace).unwrap();
        let cross_env_file = NamedTempFile::new().unwrap();
        std::fs::write(cross_env_file.path(), cross_env_trace).unwrap();

        let result = merge_cross_env_traces(
            lldb_file.path().to_str().unwrap(),
            cross_env_file.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true when merged

        // Verify merged content
        let merged = std::fs::read_to_string(lldb_file.path()).unwrap();
        let merged_parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        let merged_json = merged_parsed.get("calls").unwrap().as_array().unwrap();

        // Find the EVM call
        let evm_call = merged_json
            .iter()
            .find(|c| {
                c.get("function")
                    .and_then(|f| f.as_str())
                    .map(|f| f.starts_with("[EVM]"))
                    .unwrap_or(false)
            })
            .unwrap();

        // Verify error fields are propagated
        assert_eq!(evm_call.get("error").unwrap().as_bool().unwrap(), true);
        assert_eq!(
            evm_call.get("error_message").unwrap().as_str().unwrap(),
            "Arithmetic overflow"
        );
    }
}

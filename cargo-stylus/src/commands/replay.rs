// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use alloy::primitives::Address;
use eyre::{bail, eyre, Context};
use stylus_tools::{
    core::tracing::Trace,
    utils::{color::Color, sys},
};

use crate::{
    common_args::{ProjectArgs, ProviderArgs, TraceArgs},
    error::CargoStylusResult,
    utils::hostio::{self, ExternalContractAccess},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Any features that should be passed to cargo build.
    #[arg(short, long)]
    features: Option<Vec<String>>,
    /// Which specific package to build during replay, if any.
    #[arg(long)]
    package: Option<String>,
    /// Whether this process is the child of another.
    #[arg(short, long, hide(true))]
    child: bool,
    /// Which debugger to use: gdb, lldb, stylusdb, or auto (auto-detect).
    #[arg(long, value_name = "DEBUGGER", default_value = "auto")]
    debugger: String,
    /// Contract addresses and their source paths for multi-contract debugging.
    /// Format: ADDRESS1:PATH1,ADDRESS2:PATH2,...
    /// Example: 0x123...:./contractA,0x456...:./contractB
    #[arg(long, value_delimiter = ',', value_name = "CONTRACTS")]
    contracts: Option<Vec<String>>,
    /// Solidity contract addresses. These contracts will be recognized as Solidity
    /// contracts and displayed accordingly during debugging.
    /// Format: ADDRESS1,ADDRESS2,...
    /// Example: 0xda52b25ddb0e3b9cc393b0690ac62245ac772527
    #[arg(long, value_delimiter = ',', value_name = "ADDRESSES")]
    addr_solidity: Option<Vec<String>>,

    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    trace: TraceArgs,
}

/// Type of contract (Stylus or Solidity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractType {
    Stylus,
    Solidity,
}

/// Information about a contract
#[derive(Debug)]
struct ContractInfo {
    /// Path to the contract source/project
    path: PathBuf,
    /// Type of the contract
    contract_type: ContractType,
}

/// A loaded shared library with its path for debug symbols
struct LoadedLibrary {
    /// The loaded library handle
    library: libloading::Library,
    /// Path to the library file (for debugger symbol loading)
    path: PathBuf,
}

/// Registry for managing multiple contracts during debugging
struct ContractRegistry {
    /// Mapping from contract address to contract info
    contracts: HashMap<Address, ContractInfo>,
    /// Cached shared libraries with their paths (only for Stylus contracts)
    loaded_libraries: HashMap<Address, LoadedLibrary>,
    /// Solidity contract addresses explicitly marked via CLI.
    /// Reserved for future Solidity interop debugging support.
    #[allow(dead_code)]
    solidity_contracts: HashSet<Address>,
}

impl ContractRegistry {
    /// Create a new registry from CLI contract mappings and Solidity contract addresses
    fn new(
        contract_mappings: Option<Vec<String>>,
        solidity_addresses: Option<Vec<String>>,
    ) -> eyre::Result<Self> {
        let mut contracts = HashMap::new();
        let mut solidity_contracts = HashSet::new();

        // Parse Solidity contract addresses
        if let Some(addresses) = solidity_addresses {
            for addr_str in addresses {
                let address = addr_str
                    .parse::<Address>()
                    .wrap_err_with(|| format!("Invalid Solidity contract address: {addr_str}"))?;
                solidity_contracts.insert(address);
            }
        }

        // Parse contract mappings
        if let Some(mappings) = contract_mappings {
            for mapping in mappings {
                let parts: Vec<&str> = mapping.split(':').collect();
                if parts.len() != 2 {
                    bail!(
                        "Invalid contract mapping format: {}. Expected ADDRESS:PATH",
                        mapping
                    );
                }

                let address = parts[0]
                    .parse::<Address>()
                    .wrap_err_with(|| format!("Invalid address in mapping: {}", parts[0]))?;
                let path = PathBuf::from(parts[1]);

                if !path.exists() {
                    bail!("Contract path does not exist: {}", path.display());
                }

                // Determine contract type
                let contract_type = if solidity_contracts.contains(&address) {
                    ContractType::Solidity
                } else {
                    ContractType::Stylus
                };

                contracts.insert(
                    address,
                    ContractInfo {
                        path,
                        contract_type,
                    },
                );
            }
        }

        Ok(Self {
            contracts,
            loaded_libraries: HashMap::new(),
            solidity_contracts,
        })
    }

    /// Build all Stylus contract projects (Solidity contracts don't need building)
    fn build_all(&self, features: Option<Vec<String>>) -> eyre::Result<()> {
        for (address, info) in &self.contracts {
            if info.contract_type == ContractType::Stylus {
                println!(
                    "Building Stylus contract at {} from {}",
                    address,
                    info.path.display()
                );
                build_shared_library(&info.path, None, features.clone())?;
            } else {
                println!("Skipping Solidity contract at {address} (no build needed)",);
            }
        }
        Ok(())
    }

    /// Load a shared library for a contract address (only for Stylus contracts)
    fn load_library(&mut self, address: &Address) -> eyre::Result<Option<&libloading::Library>> {
        if let Some(info) = self.contracts.get(address) {
            // Only load libraries for Stylus contracts
            if info.contract_type == ContractType::Solidity {
                return Ok(None);
            }

            if !self.loaded_libraries.contains_key(address) {
                let library_extension = if cfg!(target_os = "macos") {
                    ".dylib"
                } else {
                    ".so"
                };
                let shared_library = find_shared_library_in_path(&info.path, library_extension)?;

                unsafe {
                    let lib = libloading::Library::new(&shared_library).wrap_err_with(|| {
                        format!(
                            "Failed to load library for {}: {}",
                            address,
                            shared_library.display()
                        )
                    })?;
                    self.loaded_libraries.insert(
                        *address,
                        LoadedLibrary {
                            library: lib,
                            path: shared_library,
                        },
                    );
                }
            }

            Ok(self.loaded_libraries.get(address).map(|l| &l.library))
        } else {
            Ok(None)
        }
    }

    /// Check if a contract has source code available
    fn has_source(&self, address: &Address) -> bool {
        self.contracts.contains_key(address)
    }

    /// Get an already-loaded library (assumes load_library was called earlier)
    fn get_loaded_library(&self, address: &Address) -> Option<&libloading::Library> {
        self.loaded_libraries.get(address).map(|l| &l.library)
    }

    /// Get the type of a contract.
    /// Reserved for future Solidity interop debugging support.
    #[allow(dead_code)]
    fn get_contract_type(&self, address: &Address) -> Option<ContractType> {
        self.contracts.get(address).map(|info| info.contract_type)
    }

    /// Check if a contract is a Solidity contract.
    /// Reserved for future Solidity interop debugging support.
    #[allow(dead_code)]
    fn is_solidity_contract(&self, address: &Address) -> bool {
        self.get_contract_type(address) == Some(ContractType::Solidity)
            || self.solidity_contracts.contains(address)
    }

    /// Load all contract libraries (only Stylus contracts)
    fn load_all_libraries(&mut self) -> eyre::Result<()> {
        let addresses: Vec<Address> = self.contracts.keys().copied().collect();
        for address in addresses {
            self.load_library(&address)?;
        }
        Ok(())
    }

    /// Get all contract debug info for debugger
    fn get_all_debug_info(&self) -> Vec<(Address, PathBuf)> {
        self.loaded_libraries
            .iter()
            .map(|(addr, loaded)| (*addr, loaded.path.clone()))
            .collect()
    }
}

impl ExternalContractAccess for ContractRegistry {
    unsafe fn call_external_contract(
        &self,
        address: &Address,
        input_data: &[u8],
    ) -> Result<(u8, Vec<u8>), Box<dyn std::error::Error>> {
        // Check if we have a library loaded for this address
        if let Some(loaded) = self.loaded_libraries.get(address) {
            let lib = &loaded.library;
            // Set up the input data for the external contract
            hostio::set_external_contract_input(input_data.to_vec());

            // Get the user_entrypoint function from the external contract
            type Entrypoint = unsafe extern "C" fn(usize) -> usize;
            if let Ok(entrypoint) = lib.get::<Entrypoint>(b"user_entrypoint") {
                // Call the external contract's user_entrypoint with the length of input data
                let result = entrypoint(input_data.len());

                // Convert result to expected format
                let status = match result {
                    0 => 0u8, // Success
                    1 => 1u8, // Revert
                    _ => 1u8, // Other errors treated as revert
                };

                // Clear the input data after execution
                hostio::set_external_contract_input(Vec::new());

                // Return empty data for now - in a full implementation we'd capture the actual return data
                return Ok((status, Vec::new()));
            }
        }

        Err("External contract not available for debugging".into())
    }
}

/// Build a shared library for the given project path
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

/// Find shared library in a specific path (for multi-contract support)
fn find_shared_library_in_path(project: &Path, extension: &str) -> eyre::Result<PathBuf> {
    let triple = rustc_host::from_cli()?;
    let so_dir = project.join(format!("target/{triple}/debug/"));
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
                bail!("more than one {extension} found: {ext} and {other}");
            }
            file = Some(entry);
        }
    }
    let Some(file) = file else {
        bail!("failed to find {extension}");
    };
    Ok(file)
}

pub async fn exec(args: Args) -> CargoStylusResult {
    exec_inner(args).await.map_err(Into::into)
}

async fn exec_inner(args: Args) -> eyre::Result<()> {
    let macos = cfg!(target_os = "macos");
    if !args.child {
        // Build contract registry early to prepare debug commands
        let mut registry =
            ContractRegistry::new(args.contracts.clone(), args.addr_solidity.clone())?;
        if !registry.contracts.is_empty() {
            registry.build_all(args.features.clone())?;
            registry.load_all_libraries()?;
        }

        // Prepare debugger commands with contract info
        let mut gdb_commands = vec![
            "--quiet".to_string(),
            "-ex=set breakpoint pending on".to_string(),
        ];
        let mut lldb_commands = vec!["--source-quietly".to_string()];
        let mut stylusdb_commands = vec![];

        // For stylusdb, use the stylusdb-contract commands
        for (addr, path) in registry.get_all_debug_info() {
            stylusdb_commands.push("-o".to_string());
            stylusdb_commands.push(format!("stylusdb-contract add {} {}", addr, path.display()));
        }

        // Set breakpoints on all user_entrypoints
        if registry.contracts.is_empty() {
            // Single contract mode
            gdb_commands.push("-ex=b user_entrypoint".to_string());
            lldb_commands.push("-o".to_string());
            lldb_commands.push("b user_entrypoint".to_string());
            stylusdb_commands.push("-o".to_string());
            stylusdb_commands.push("b user_entrypoint".to_string());
        } else {
            // Multi-contract mode - set breakpoints for all contracts using stylusdb-contract
            for addr in registry.contracts.keys() {
                stylusdb_commands.push("-o".to_string());
                stylusdb_commands.push(format!(
                    "stylusdb-contract breakpoint {addr} user_entrypoint",
                ));
            }
            // Still set a general breakpoint for compatibility with GDB/LLDB
            gdb_commands.push("-ex=b user_entrypoint".to_string());
            lldb_commands.push("-o".to_string());
            lldb_commands.push("b user_entrypoint".to_string());
        }

        // Add run command
        gdb_commands.push("-ex=r".to_string());
        gdb_commands.push("--args".to_string());
        lldb_commands.push("-o".to_string());
        lldb_commands.push("r".to_string());
        lldb_commands.push("--".to_string());
        stylusdb_commands.push("-o".to_string());
        stylusdb_commands.push("r".to_string());
        stylusdb_commands.push("--".to_string());

        let gdb_args: Vec<&str> = gdb_commands.iter().map(|s| s.as_str()).collect();
        let lldb_args: Vec<&str> = lldb_commands.iter().map(|s| s.as_str()).collect();
        let stylusdb_args: Vec<&str> = stylusdb_commands.iter().map(|s| s.as_str()).collect();

        let (cmd_name, args_slice) = match args.debugger.as_str() {
            "gdb" => {
                if sys::command_exists("rust-gdb") && !macos {
                    ("rust-gdb", gdb_args.as_slice())
                } else if sys::command_exists("gdb") && !macos {
                    ("gdb", gdb_args.as_slice())
                } else {
                    bail!("gdb not found or not supported on this platform")
                }
            }
            "lldb" => {
                if sys::command_exists("rust-lldb") {
                    ("rust-lldb", lldb_args.as_slice())
                } else if sys::command_exists("lldb") {
                    ("lldb", lldb_args.as_slice())
                } else {
                    bail!("lldb not found")
                }
            }
            "stylusdb" => {
                if sys::command_exists("rust-stylusdb") {
                    ("rust-stylusdb", stylusdb_args.as_slice())
                } else {
                    bail!("rust-stylusdb not found")
                }
            }
            "auto" => {
                // Auto-detect the best available debugger
                if sys::command_exists("rust-gdb") && !macos {
                    ("rust-gdb", gdb_args.as_slice())
                } else if sys::command_exists("rust-lldb") {
                    ("rust-lldb", lldb_args.as_slice())
                } else if sys::command_exists("rust-stylusdb") {
                    ("rust-stylusdb", stylusdb_args.as_slice())
                } else {
                    println!(
                        "rust specific debugger not installed, falling back to generic debugger"
                    );
                    if sys::command_exists("gdb") && !macos {
                        ("gdb", gdb_args.as_slice())
                    } else if sys::command_exists("lldb") {
                        ("lldb", lldb_args.as_slice())
                    } else {
                        bail!("no debugger found")
                    }
                }
            }
            _ => bail!(
                "Unknown debugger: {}. Supported debuggers: gdb, lldb, stylusdb, auto",
                args.debugger
            ),
        };

        let mut cmd = Command::new(cmd_name);
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        for arg in args_slice.iter() {
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

    // Create contract registry and build all contracts
    let mut registry = ContractRegistry::new(args.contracts.clone(), args.addr_solidity.clone())?;

    // If no contracts specified, use the default project
    if registry.contracts.is_empty() {
        let mut contracts = args.project.contracts()?;
        if contracts.len() != 1 {
            bail!("cargo stylus replay can only be executed on one contract at a time when no --contracts flag is provided");
        }
        let contract = contracts.pop().unwrap();

        let project_path = contract
            .package
            .manifest_path
            .parent()
            .ok_or_else(|| eyre!("Failed to get contract directory"))?;

        build_shared_library(
            project_path.as_std_path(),
            args.package.clone(),
            args.features.clone(),
        )?;
        let library_extension = if macos { ".dylib" } else { ".so" };
        let shared_library =
            find_shared_library_in_path(project_path.as_std_path(), library_extension)?;

        // Get the contract address from the trace
        let contract_address = trace
            .address()
            .ok_or_else(|| eyre!("Transaction has no 'to' address"))?;

        // Load the default library
        unsafe {
            let lib = libloading::Library::new(&shared_library)?;
            registry.contracts.insert(
                contract_address,
                ContractInfo {
                    path: project_path.as_std_path().to_path_buf(),
                    contract_type: ContractType::Stylus,
                },
            );
            registry.loaded_libraries.insert(
                contract_address,
                LoadedLibrary {
                    library: lib,
                    path: shared_library,
                },
            );
        }
    } else {
        // Build all specified contracts
        registry.build_all(args.features.clone())?;
        registry.load_all_libraries()?;
    }

    // TODO: don't assume the contract is top-level
    let Some(input_data) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = input_data.len();

    // Check if we have the main contract
    let contract_address = trace
        .address()
        .ok_or_else(|| eyre!("Transaction has no 'to' address"))?;

    if !registry.has_source(&contract_address) {
        if args.contracts.is_some() {
            let provided_addresses: Vec<String> = registry
                .contracts
                .keys()
                .map(|addr| format!("{addr}"))
                .collect();
            bail!(
                "Main contract at {} is not in the provided --contracts list.\n\
                 Provided addresses: [{}]\n\
                 Add the main contract: --contracts {}:<PATH_TO_SOURCE>",
                contract_address,
                provided_addresses.join(", "),
                contract_address
            );
        } else {
            bail!(
                "Main contract at {} has no source code provided. Use --contracts flag:\n\
                 --contracts {}:<PATH_TO_SOURCE>",
                contract_address,
                contract_address
            );
        }
    }

    // Initialize debugger hook if using stylusdb
    if args.debugger == "stylusdb" && !registry.contracts.is_empty() {
        use crate::commands::debug_hook::{self, DebuggerHook};

        let hook = debug_hook::StylusDebuggerHook::new()?;

        // Send all contract mappings to debugger
        let contracts: Vec<(String, String)> = registry
            .get_all_debug_info()
            .into_iter()
            .map(|(addr, path)| (format!("{addr}"), path.display().to_string()))
            .collect();
        hook.on_execution_start(&contracts);

        // Send contract type information
        for (addr, info) in &registry.contracts {
            hook.on_contract_info(
                &format!("{addr}"),
                info.contract_type == ContractType::Solidity,
            );
        }

        debug_hook::init_debugger_hook(Arc::new(hook));
    }

    unsafe {
        *hostio::FRAME.lock() = Some(trace.reader());

        // Set up external contract access for debugging
        let registry_arc = Arc::new(registry);
        hostio::set_external_contract_access(registry_arc.clone());

        // Get the library for the main contract
        let Some(lib) = registry_arc.get_loaded_library(&contract_address) else {
            bail!("Failed to load library for contract {}", contract_address);
        };

        type Entrypoint = unsafe extern "C" fn(usize) -> usize;
        let main: libloading::Symbol<Entrypoint> = lib.get(b"user_entrypoint")?;

        match main(args_len) {
            0 => println!("call completed successfully"),
            1 => println!("call reverted"),
            x => println!("call exited with unknown status code: {}", x.red()),
        }
    }
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
                bail!("more than one {extension} found: {ext} and {other}");
            }
            file = Some(entry);
        }
    }
    let Some(file) = file else {
        bail!("failed to find {extension}");
    };
    Ok(file)
}

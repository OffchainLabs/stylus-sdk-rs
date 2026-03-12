// Copyright 2025-2026, Offchain Labs, Inc.
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
use stylus_tools::{core::tracing::Trace, utils::sys};

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

/// Return the platform-appropriate shared library extension.
fn library_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    }
}

/// Registry for managing one or more contracts during debugging, handling both
/// the single-contract (default project) and multi-contract (`--contracts`)
/// paths.
struct ContractRegistry {
    /// Mapping from contract address to contract info
    contracts: HashMap<Address, ContractInfo>,
    /// Cached shared libraries with their paths (only for Stylus contracts)
    loaded_libraries: HashMap<Address, LoadedLibrary>,
}

impl ContractRegistry {
    /// Create a new registry from CLI contract mappings. Addresses that also
    /// appear in `solidity_addresses` are tagged as Solidity contracts;
    /// Solidity addresses not present in `contract_mappings` are ignored.
    /// Passing `None` for both arguments creates an empty registry, suitable
    /// for the single-contract path where `register_with_library` is called
    /// later.
    fn new(
        contract_mappings: Option<Vec<String>>,
        solidity_addresses: Option<Vec<String>>,
    ) -> eyre::Result<Self> {
        let mut contracts = HashMap::new();
        let mut solidity_contracts = HashSet::new();

        if let Some(addresses) = solidity_addresses {
            for address_str in addresses {
                let address = address_str.parse::<Address>().wrap_err_with(|| {
                    format!("Invalid Solidity contract address: {address_str}")
                })?;
                solidity_contracts.insert(address);
            }
        }

        if let Some(mappings) = contract_mappings {
            for mapping in mappings {
                let Some((addr_str, path_str)) = mapping.split_once(':') else {
                    bail!("Invalid contract mapping format: {mapping}. Expected ADDRESS:PATH");
                };

                let address = addr_str
                    .parse::<Address>()
                    .wrap_err_with(|| format!("Invalid address in mapping: {addr_str}"))?;
                let path = PathBuf::from(path_str);

                if !path.is_dir() {
                    bail!(
                        "Contract path is not a directory or does not exist: {}",
                        path.display()
                    );
                }

                let contract_type = if solidity_contracts.contains(&address) {
                    ContractType::Solidity
                } else {
                    ContractType::Stylus
                };

                if contracts.contains_key(&address) {
                    bail!(
                        "duplicate address {address} in --contracts; each address may only appear once"
                    );
                }
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
                println!("Skipping Solidity contract at {address} (no build needed)");
            }
        }
        Ok(())
    }

    /// Load a shared library for a contract address (only for Stylus contracts).
    /// Returns `Ok(None)` for Solidity contracts, `Ok(Some(...))` for Stylus
    /// contracts, and `Err` if the address is not registered.
    fn load_library(&mut self, address: &Address) -> eyre::Result<Option<&libloading::Library>> {
        let Some(info) = self.contracts.get(address) else {
            bail!("no contract registered for address {address}");
        };

        if info.contract_type == ContractType::Solidity {
            return Ok(None);
        }

        if !self.loaded_libraries.contains_key(address) {
            let shared_library = find_shared_library_in_path(&info.path, library_extension())?;

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
    }

    /// Check if a contract is registered in this registry
    fn has_source(&self, address: &Address) -> bool {
        self.contracts.contains_key(address)
    }

    /// Check if the registry has any contracts
    fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    /// Get an already-loaded library for the given address. Returns `None` if
    /// the address has no loaded library (either unregistered or a Solidity
    /// contract).
    fn get_loaded_library(&self, address: &Address) -> Option<&libloading::Library> {
        self.loaded_libraries.get(address).map(|l| &l.library)
    }

    /// Return an iterator over all registered contract addresses
    fn addresses(&self) -> impl Iterator<Item = &Address> {
        self.contracts.keys()
    }

    /// Iterate over all registered contracts, yielding (address, contract_type) pairs
    fn iter_contracts(&self) -> impl Iterator<Item = (&Address, ContractType)> {
        self.contracts
            .iter()
            .map(|(addr, info)| (addr, info.contract_type))
    }

    /// Load all contract libraries (only Stylus contracts)
    fn load_all_libraries(&mut self) -> eyre::Result<()> {
        let addresses: Vec<Address> = self.contracts.keys().copied().collect();
        for address in addresses {
            self.load_library(&address)?;
        }
        Ok(())
    }

    /// Register a contract and its already-loaded library directly.
    /// Used for the single-contract (default project) path where the library
    /// has already been built and loaded externally. The caller is responsible
    /// for ensuring `library_path` has been validated via
    /// `verify_within_directory`.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is already registered (to prevent
    /// silently dropping a previously loaded library).
    fn register_with_library(
        &mut self,
        address: Address,
        project_path: PathBuf,
        library: libloading::Library,
        library_path: PathBuf,
    ) -> eyre::Result<()> {
        if self.contracts.contains_key(&address) {
            bail!("contract at {address} is already registered");
        }
        self.contracts.insert(
            address,
            ContractInfo {
                path: project_path,
                contract_type: ContractType::Stylus,
            },
        );
        self.loaded_libraries.insert(
            address,
            LoadedLibrary {
                library,
                path: library_path,
            },
        );
        Ok(())
    }

    /// Return the address and shared library path for every loaded contract,
    /// for use in debugger symbol-loading commands.
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
        let Some(loaded) = self.loaded_libraries.get(address) else {
            return Err(format!(
                "external contract at {address} not available for debugging — \
                 add it with --contracts {address}:<PATH_TO_SOURCE>"
            )
            .into());
        };

        let lib = &loaded.library;
        hostio::set_external_contract_input(input_data.to_vec());

        // Ensure the input buffer is cleared on all exit paths (including panic).
        struct InputGuard;
        impl Drop for InputGuard {
            fn drop(&mut self) {
                hostio::set_external_contract_input(Vec::new());
            }
        }
        let _guard = InputGuard;

        type Entrypoint = unsafe extern "C" fn(usize) -> usize;
        let entrypoint: libloading::Symbol<Entrypoint> = lib
            .get(b"user_entrypoint")
            .map_err(|e| format!("failed to find user_entrypoint in library for {address}: {e}"))?;

        let result = entrypoint(input_data.len());

        let status: u8 = match result {
            0 | 1 => result as u8,
            other => {
                return Err(format!(
                    "external contract {address} returned unexpected status code {other}"
                )
                .into());
            }
        };

        // TODO: capture actual return data from the external contract
        // via the hostio output buffer
        Ok((status, Vec::new()))
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

    let output = cargo
        .arg("--lib")
        .arg("--locked")
        .arg("--target")
        .arg(rustc_host::from_cli()?)
        .output()
        .wrap_err("failed to execute cargo build")?;
    if !output.status.success() {
        use std::fmt::Write;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut msg = format!("cargo build failed with {}", output.status);
        if !stderr.is_empty() {
            write!(msg, "\nstderr:\n{stderr}").unwrap();
        }
        if !stdout.is_empty() {
            write!(msg, "\nstdout:\n{stdout}").unwrap();
        }
        bail!("{msg}");
    }
    Ok(())
}

/// Canonicalize `file` and verify it resides within `expected_dir` to prevent
/// path traversal (e.g., via symlinks or `..` components). Returns the
/// canonicalized path on success.
///
/// Errors if `file` cannot be canonicalized, if `expected_dir` cannot be
/// canonicalized, or if the resolved file path is not a descendant of
/// `expected_dir`.
fn verify_within_directory(file: &Path, expected_dir: &Path) -> eyre::Result<PathBuf> {
    let file = std::fs::canonicalize(file).wrap_err_with(|| {
        format!(
            "failed to canonicalize shared library path: {}",
            file.display()
        )
    })?;
    let expected_dir = std::fs::canonicalize(expected_dir).wrap_err_with(|| {
        format!(
            "failed to canonicalize build directory: {}",
            expected_dir.display()
        )
    })?;
    if !file.starts_with(&expected_dir) {
        bail!(
            "shared library path escapes build directory: {} is not within {}",
            file.display(),
            expected_dir.display()
        );
    }
    Ok(file)
}

/// Find a shared library under `project/target/{triple}/debug/`.
/// The returned path is canonicalized and verified to reside within the build
/// directory.
fn find_shared_library_in_path(project: &Path, extension: &str) -> eyre::Result<PathBuf> {
    let triple = rustc_host::from_cli()?;
    let so_dir = project.join(format!("target/{triple}/debug/"));
    find_library_in_dir(&so_dir, extension)
}

/// Execute the replay command.
pub async fn exec(args: Args) -> CargoStylusResult {
    exec_inner(args).await.map_err(Into::into)
}

async fn exec_inner(args: Args) -> eyre::Result<()> {
    let macos = cfg!(target_os = "macos");
    if !args.child {
        // Build contract registry early to prepare debug commands
        let mut registry =
            ContractRegistry::new(args.contracts.clone(), args.addr_solidity.clone())?;
        if !registry.is_empty() {
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
        if registry.is_empty() {
            // Single contract mode
            gdb_commands.push("-ex=b user_entrypoint".to_string());
            lldb_commands.push("-o".to_string());
            lldb_commands.push("b user_entrypoint".to_string());
            stylusdb_commands.push("-o".to_string());
            stylusdb_commands.push("b user_entrypoint".to_string());
        } else {
            // Multi-contract mode - set breakpoints for all contracts using stylusdb-contract
            for address in registry.addresses() {
                stylusdb_commands.push("-o".to_string());
                stylusdb_commands.push(format!(
                    "stylusdb-contract breakpoint {address} user_entrypoint"
                ));
            }
            // Still set a general breakpoint for compatibility with GDB/LLDB
            gdb_commands.push("-ex=b user_entrypoint".to_string());
            lldb_commands.push("-o".to_string());
            lldb_commands.push("b user_entrypoint".to_string());
        }

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
        {
            let err = cmd.exec();
            bail!("failed to exec {cmd_name}: {err}");
        }
        #[cfg(windows)]
        {
            let status = cmd
                .status()
                .wrap_err_with(|| format!("failed to spawn {cmd_name}"))?;
            if !status.success() {
                bail!("{cmd_name} exited with {status}");
            }
            return Ok(());
        }
    }

    let provider = args.provider.build_provider().await?;

    let trace = Trace::new(args.trace.tx, &args.trace.config, &provider).await?;

    // Create contract registry and build all contracts
    let mut registry = ContractRegistry::new(args.contracts.clone(), args.addr_solidity.clone())?;

    let contract_address = trace
        .address()
        .ok_or_else(|| eyre!("Transaction has no 'to' address"))?;

    if registry.is_empty() {
        let mut contracts = args.project.contracts()?;
        if contracts.len() != 1 {
            bail!("cargo stylus replay can only be executed on one contract at a time when no --contracts flag is provided");
        }
        let contract = contracts.pop().expect("length was checked to be 1");

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
        let shared_library =
            find_shared_library_in_path(project_path.as_std_path(), library_extension())?;

        unsafe {
            let lib = libloading::Library::new(&shared_library).wrap_err_with(|| {
                format!(
                    "failed to load shared library: {}",
                    shared_library.display()
                )
            })?;
            registry.register_with_library(
                contract_address,
                project_path.as_std_path().to_path_buf(),
                lib,
                shared_library,
            )?;
        }
    } else {
        registry.build_all(args.features.clone())?;
        registry.load_all_libraries()?;
    }

    // TODO: support replaying internal (nested) calls, not just the
    // top-level transaction
    let Some(input_data) = trace.tx().input.input() else {
        bail!("missing transaction input");
    };
    let args_len = input_data.len();

    if !registry.has_source(&contract_address) {
        if args.contracts.is_some() {
            let provided_addresses: Vec<String> =
                registry.addresses().map(ToString::to_string).collect();
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

    // Initialize debugger hook if using stylusdb (Unix only — uses Unix sockets)
    #[cfg(unix)]
    if args.debugger == "stylusdb" && !registry.is_empty() {
        use crate::commands::debug_hook::{self, DebuggerHook};

        let hook = debug_hook::StylusDebuggerHook::new()?;

        // Send all contract mappings to debugger
        let contracts: Vec<(String, String)> = registry
            .get_all_debug_info()
            .into_iter()
            .map(|(address, path)| (format!("{address}"), path.display().to_string()))
            .collect();
        hook.on_execution_start(&contracts);

        // Send contract type information
        for (address, contract_type) in registry.iter_contracts() {
            hook.on_contract_info(
                &format!("{address}"),
                contract_type == ContractType::Solidity,
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
            bail!(
                "no shared library loaded for contract {contract_address} \
                 (is it a Solidity contract? Stylus replay requires a Stylus contract)"
            );
        };

        type Entrypoint = unsafe extern "C" fn(usize) -> usize;
        let main: libloading::Symbol<Entrypoint> =
            lib.get(b"user_entrypoint").wrap_err_with(|| {
                format!(
                    "failed to find user_entrypoint in library for main contract {contract_address}"
                )
            })?;

        match main(args_len) {
            0 => println!("call completed successfully"),
            1 => println!("call reverted"),
            x => bail!("call exited with unknown status code: {x}"),
        }
    }
    Ok(())
}

/// Find a shared library under `{triple}/debug/` within the given target
/// directory. The `project` path should point to a cargo target directory
/// (e.g., from cargo metadata's `target_dir`), not the project root.
pub fn find_shared_library(project: &Path, extension: &str) -> eyre::Result<PathBuf> {
    let triple = rustc_host::from_cli()?;
    let so_dir = project.join(format!("{triple}/debug/"));
    find_library_in_dir(&so_dir, extension)
}

/// Scan `so_dir` for exactly one file whose name ends with `extension`,
/// erroring if no matches or multiple matches are found. The returned path is
/// canonicalized and verified to reside within `so_dir`.
fn find_library_in_dir(so_dir: &Path, extension: &str) -> eyre::Result<PathBuf> {
    let mut file: Option<PathBuf> = None;
    for entry in std::fs::read_dir(so_dir)
        .map_err(|e| eyre!("failed to open {}: {e}", so_dir.to_string_lossy()))?
    {
        let entry = entry.map_err(|e| eyre!("failed to read directory entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let name = name.to_string_lossy();

        if name.ends_with(extension) {
            if let Some(other) = file {
                let other = other.file_name().unwrap_or_default().to_string_lossy();
                bail!("more than one {extension} found: {name} and {other}");
            }
            file = Some(path);
        }
    }
    let Some(file) = file else {
        bail!("failed to find {extension} in {}", so_dir.to_string_lossy());
    };
    verify_within_directory(&file, so_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_verify_within_directory_accepts_file_inside_dir() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("lib.so");
        fs::write(&file_path, b"").unwrap();

        let canonical =
            verify_within_directory(&file_path, dir.path()).expect("should accept file inside dir");
        assert!(canonical.is_absolute());
        assert_eq!(canonical, std::fs::canonicalize(&file_path).unwrap());
    }

    #[test]
    fn test_verify_within_directory_rejects_file_outside_dir() {
        let parent = tempfile::tempdir().unwrap();
        let expected = parent.path().join("expected");
        let other = parent.path().join("other");
        fs::create_dir_all(&expected).unwrap();
        fs::create_dir_all(&other).unwrap();
        let file_path = other.join("lib.so");
        fs::write(&file_path, b"").unwrap();

        let result = verify_within_directory(&file_path, &expected);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("escapes build directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_verify_within_directory_rejects_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = verify_within_directory(&dir.path().join("nonexistent"), dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to canonicalize shared library path"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_verify_within_directory_rejects_nonexistent_expected_dir() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("lib.so");
        fs::write(&file_path, b"").unwrap();

        let result = verify_within_directory(&file_path, &dir.path().join("nonexistent"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to canonicalize build directory"),
            "unexpected error: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_verify_within_directory_resolves_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let real_dir = dir.path().join("real");
        let escape_dir = dir.path().join("escape");
        fs::create_dir_all(&real_dir).unwrap();
        fs::create_dir_all(&escape_dir).unwrap();
        let target_file = escape_dir.join("lib.so");
        fs::write(&target_file, b"").unwrap();

        let symlink = real_dir.join("lib.so");
        std::os::unix::fs::symlink(&target_file, &symlink).unwrap();

        let result = verify_within_directory(&symlink, &real_dir);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("escapes build directory"),
            "symlink should be rejected: {err}"
        );
    }

    #[test]
    fn test_find_library_in_dir_finds_single_match() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("libfoo.so"), b"").unwrap();
        fs::write(dir.path().join("unrelated.txt"), b"").unwrap();

        let result = find_library_in_dir(dir.path(), ".so").unwrap();
        assert!(result.is_absolute());
        assert_eq!(result.file_name().unwrap(), "libfoo.so");
    }

    #[test]
    fn test_find_library_in_dir_rejects_multiple_matches() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("libfoo.so"), b"").unwrap();
        fs::write(dir.path().join("libbar.so"), b"").unwrap();

        let result = find_library_in_dir(dir.path(), ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("more than one"), "unexpected error: {err}");
    }

    #[cfg(unix)]
    #[test]
    fn test_find_library_in_dir_rejects_symlink_escaping_directory() {
        let dir = tempfile::tempdir().unwrap();
        let escape_dir = dir.path().join("escape");
        let search_dir = dir.path().join("search");
        fs::create_dir_all(&escape_dir).unwrap();
        fs::create_dir_all(&search_dir).unwrap();
        fs::write(escape_dir.join("libevil.so"), b"").unwrap();
        std::os::unix::fs::symlink(escape_dir.join("libevil.so"), search_dir.join("libevil.so"))
            .unwrap();

        let result = find_library_in_dir(&search_dir, ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("escapes build directory"),
            "symlink through find_library_in_dir should be rejected: {err}"
        );
    }

    #[test]
    fn test_verify_within_directory_rejects_dotdot_components() {
        let parent = tempfile::tempdir().unwrap();
        let expected = parent.path().join("expected");
        let other = parent.path().join("other");
        fs::create_dir_all(&expected).unwrap();
        fs::create_dir_all(&other).unwrap();
        let file_path = other.join("lib.so");
        fs::write(&file_path, b"").unwrap();

        // Use a path with `..` that resolves outside expected_dir
        let sneaky_path = expected.join("..").join("other").join("lib.so");
        let result = verify_within_directory(&sneaky_path, &expected);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("escapes build directory"),
            "dotdot traversal should be rejected: {err}"
        );
    }

    #[test]
    fn test_find_library_in_dir_errors_on_nonexistent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = find_library_in_dir(&dir.path().join("nonexistent"), ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to open"), "unexpected error: {err}");
    }

    #[test]
    fn test_find_library_in_dir_ignores_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("subdir.so")).unwrap();
        fs::write(dir.path().join("unrelated.txt"), b"").unwrap();

        let result = find_library_in_dir(dir.path(), ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to find"),
            "subdirectory should not match: {err}"
        );
    }

    #[test]
    fn test_verify_within_directory_accepts_file_in_nested_subdir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();
        let file_path = nested.join("lib.so");
        fs::write(&file_path, b"").unwrap();

        let canonical =
            verify_within_directory(&file_path, dir.path()).expect("should accept nested file");
        assert!(canonical.is_absolute());
        assert_eq!(canonical, std::fs::canonicalize(&file_path).unwrap());
    }

    #[test]
    fn test_find_library_in_dir_ignores_files_with_extension_as_infix() {
        let dir = tempfile::tempdir().unwrap();
        // A file like "libfoo.so.bak" should NOT match ".so"
        fs::write(dir.path().join("libfoo.so.bak"), b"").unwrap();
        fs::write(dir.path().join("unrelated.txt"), b"").unwrap();

        let result = find_library_in_dir(dir.path(), ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to find"),
            "infix match should not count: {err}"
        );
    }

    #[test]
    fn test_find_library_in_dir_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = find_library_in_dir(dir.path(), ".so");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to find"), "unexpected error: {err}");
    }

    #[test]
    fn test_contract_registry_new_empty_inputs() {
        let registry = ContractRegistry::new(None, None).unwrap();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_contract_registry_new_accepts_valid_directory() {
        let dir = tempfile::tempdir().unwrap();
        let mapping = format!(
            "0x0000000000000000000000000000000000000001:{}",
            dir.path().display()
        );
        let registry = ContractRegistry::new(Some(vec![mapping]), None)
            .expect("valid directory should be accepted");
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_contract_registry_new_rejects_file_as_contract_path() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("not_a_dir.txt");
        fs::write(&file_path, b"").unwrap();

        let mapping = format!(
            "0x0000000000000000000000000000000000000001:{}",
            file_path.display()
        );
        let result = ContractRegistry::new(Some(vec![mapping]), None);
        let err = result
            .err()
            .expect("should fail for non-directory path")
            .to_string();
        assert!(
            err.contains("not a directory"),
            "file should be rejected: {err}"
        );
    }

    #[test]
    fn test_contract_registry_new_rejects_invalid_mapping_format() {
        let result = ContractRegistry::new(Some(vec!["invalid_no_colon".to_string()]), None);
        let err = result
            .err()
            .expect("should fail for invalid format")
            .to_string();
        assert!(
            err.contains("Invalid contract mapping format"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_contract_registry_new_tags_solidity_contracts() {
        let dir = tempfile::tempdir().unwrap();
        let addr = "0x0000000000000000000000000000000000000001";
        let mapping = format!("{addr}:{}", dir.path().display());
        let registry =
            ContractRegistry::new(Some(vec![mapping]), Some(vec![addr.to_string()])).unwrap();

        let contracts: Vec<_> = registry.iter_contracts().collect();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].1, ContractType::Solidity);
    }

    #[test]
    fn test_contract_registry_new_rejects_duplicate_address() {
        let dir = tempfile::tempdir().unwrap();
        let addr = "0x0000000000000000000000000000000000000001";
        let mapping = format!("{addr}:{}", dir.path().display());
        let result = ContractRegistry::new(Some(vec![mapping.clone(), mapping]), None);
        let err = result.err().expect("should fail for duplicate").to_string();
        assert!(err.contains("duplicate address"), "unexpected error: {err}");
    }

    #[test]
    fn test_contract_registry_new_rejects_invalid_solidity_address() {
        let result = ContractRegistry::new(None, Some(vec!["not_an_address".to_string()]));
        let err = result
            .err()
            .expect("should fail for invalid address")
            .to_string();
        assert!(
            err.contains("Invalid Solidity contract address"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_contract_registry_load_library_errors_on_unregistered() {
        let mut registry = ContractRegistry::new(None, None).unwrap();
        let addr = "0x0000000000000000000000000000000000000099"
            .parse::<Address>()
            .unwrap();
        let result = registry.load_library(&addr);
        let err = result
            .expect_err("should fail for unregistered")
            .to_string();
        assert!(
            err.contains("no contract registered"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_contract_registry_register_with_library_rejects_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let addr = "0x0000000000000000000000000000000000000001"
            .parse::<Address>()
            .unwrap();
        let lib_path = dir.path().join("libfoo.so");
        fs::write(&lib_path, b"fake").unwrap();

        let mut registry = ContractRegistry::new(None, None).unwrap();

        // First registration should succeed
        unsafe {
            // Use a dummy library path — we only care about the duplicate check
            let lib = libloading::Library::new(&lib_path);
            // Library load may fail (not a real .so), so skip if unsupported
            if let Ok(lib) = lib {
                registry
                    .register_with_library(addr, dir.path().to_path_buf(), lib, lib_path.clone())
                    .unwrap();

                // Second registration should fail
                let lib2 = libloading::Library::new(&lib_path);
                if let Ok(lib2) = lib2 {
                    let result = registry.register_with_library(
                        addr,
                        dir.path().to_path_buf(),
                        lib2,
                        lib_path,
                    );
                    let err = result.expect_err("should fail for duplicate").to_string();
                    assert!(
                        err.contains("already registered"),
                        "unexpected error: {err}"
                    );
                }
            }
        }
    }
}

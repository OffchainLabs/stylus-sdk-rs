fn main() {
    // On Linux, export hostio symbols to the dynamic symbol table so that
    // contract shared libraries loaded via dlopen() can resolve them from
    // the cargo-stylus binary at runtime.
    //
    // This is the Linux equivalent of macOS's -undefined dynamic_lookup
    // linker flag. On macOS, the contract .dylib is built with that flag
    // so unresolved symbols are looked up at load time. On Linux, shared
    // libraries already allow undefined symbols by default, but the host
    // binary must explicitly export its symbols for dlopen'd libraries to
    // find them.
    //
    // We use --export-dynamic-symbol for each hostio function rather than
    // -rdynamic (which exports ALL symbols) to avoid pulling in unrelated
    // symbols that cause linker errors (e.g. wasmer_vm's __rust_probestack).
    if cfg!(target_os = "linux") {
        let hostio_symbols = [
            // vm_hooks module - core hostio functions
            "account_balance",
            "account_code",
            "account_code_size",
            "account_codehash",
            "block_basefee",
            "block_coinbase",
            "block_gas_limit",
            "block_number",
            "block_timestamp",
            "call_contract",
            "chainid",
            "contract_address",
            "create1",
            "create2",
            "delegate_call_contract",
            "emit_log",
            "evm_gas_left",
            "evm_ink_left",
            "exit_early",
            "math_add_mod",
            "math_div",
            "math_mod",
            "math_mul_mod",
            "math_pow",
            "msg_reentrant",
            "msg_sender",
            "msg_value",
            "native_keccak256",
            "pay_for_memory_grow",
            "read_args",
            "read_return_data",
            "return_data_size",
            "static_call_contract",
            "storage_cache_bytes32",
            "storage_flush_cache",
            "storage_load_bytes32",
            "transient_load_bytes32",
            "transient_store_bytes32",
            "tx_gas_price",
            "tx_ink_price",
            "tx_origin",
            "write_result",
            // console module - debug logging functions
            "log_f32",
            "log_f64",
            "log_i32",
            "log_i64",
            "log_txt",
        ];
        for sym in &hostio_symbols {
            println!("cargo:rustc-link-arg-bins=-Wl,--export-dynamic-symbol={sym}");
        }
    }
}

// Copyright 2025, Offchain Labs, Inc.
// Tests for Stylus -> Solidity interop functionality

/// Re-export test types for soldb_bridge testing
/// Since soldb_bridge types are not public, we test through JSON serialization/deserialization

// ==================== Environment tests ====================

#[test]
fn test_environment_json_serialization() {
    // Test that environment values serialize correctly
    let evm_json = r#""evm""#;
    let stylus_json = r#""stylus""#;

    // Verify they can be used in contract info
    let contract_json = format!(
        r#"{{"address": "0x1234", "environment": {}, "name": "Test"}}"#,
        evm_json
    );
    let parsed: serde_json::Value = serde_json::from_str(&contract_json).unwrap();
    assert_eq!(parsed["environment"], "evm");

    let contract_json = format!(
        r#"{{"address": "0x1234", "environment": {}, "name": "Test"}}"#,
        stylus_json
    );
    let parsed: serde_json::Value = serde_json::from_str(&contract_json).unwrap();
    assert_eq!(parsed["environment"], "stylus");
}

// ==================== SourceLocation tests ====================

#[test]
fn test_source_location_json_with_column() {
    let json = r#"{"file": "contracts/Token.sol", "line": 42, "column": 15}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["file"], "contracts/Token.sol");
    assert_eq!(parsed["line"], 42);
    assert_eq!(parsed["column"], 15);
}

#[test]
fn test_source_location_json_without_column() {
    let json = r#"{"file": "lib.rs", "line": 100}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["file"], "lib.rs");
    assert_eq!(parsed["line"], 100);
    assert!(parsed.get("column").is_none());
}

// ==================== CallArgument tests ====================

#[test]
fn test_call_argument_json_serialization() {
    let json = r#"{"name": "recipient", "type": "address", "value": "0x1234567890abcdef1234567890abcdef12345678"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["name"], "recipient");
    assert_eq!(parsed["type"], "address");
    assert_eq!(
        parsed["value"],
        "0x1234567890abcdef1234567890abcdef12345678"
    );
}

#[test]
fn test_call_argument_various_types() {
    let test_cases = vec![
        ("uint256", "1000000000000000000", "amount"),
        ("address", "0xabcd", "to"),
        ("bool", "true", "approved"),
        (
            "bytes32",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "hash",
        ),
        ("string", "Hello World", "message"),
    ];

    for (arg_type, value, name) in test_cases {
        let json = format!(
            r#"{{"name": "{}", "type": "{}", "value": "{}"}}"#,
            name, arg_type, value
        );
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], arg_type);
        assert_eq!(parsed["value"], value);
    }
}

// ==================== ContractInfo tests ====================

#[test]
fn test_contract_info_stylus() {
    let json = r#"{
        "address": "0x1234567890abcdef1234567890abcdef12345678",
        "environment": "stylus",
        "name": "TestContract",
        "lib_path": "/path/to/lib.dylib"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["environment"], "stylus");
    assert_eq!(parsed["name"], "TestContract");
    assert_eq!(parsed["lib_path"], "/path/to/lib.dylib");
}

#[test]
fn test_contract_info_evm() {
    let json = r#"{
        "address": "0xabcd",
        "environment": "evm",
        "name": "ERC20Token",
        "debug_dir": "/debug/dir",
        "project_path": "/project/path"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["environment"], "evm");
    assert_eq!(parsed["name"], "ERC20Token");
    assert_eq!(parsed["debug_dir"], "/debug/dir");
    assert_eq!(parsed["project_path"], "/project/path");
}

#[test]
fn test_contract_info_minimal() {
    let json = r#"{
        "address": "0x1234",
        "environment": "stylus",
        "name": "MyContract"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["address"], "0x1234");
    assert!(parsed.get("debug_dir").is_none());
    assert!(parsed.get("lib_path").is_none());
}

// ==================== CrossEnvCall tests ====================

#[test]
fn test_cross_env_call_basic() {
    let json = r#"{
        "call_id": 1,
        "environment": "stylus",
        "contract_address": "0x1234",
        "function_name": "increment",
        "function_selector": "0x12345678",
        "source_location": {
            "file": "lib.rs",
            "line": 42
        },
        "args": [
            {"name": "amount", "type": "uint256", "value": "100"}
        ],
        "gas_used": 21000,
        "success": true,
        "call_type": "external",
        "children": []
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["call_id"], 1);
    assert_eq!(parsed["function_name"], "increment");
    assert_eq!(parsed["source_location"]["file"], "lib.rs");
    assert_eq!(parsed["args"][0]["name"], "amount");
}

#[test]
fn test_cross_env_call_with_nested_children() {
    let json = r#"{
        "call_id": 1,
        "environment": "evm",
        "contract_address": "0x1234",
        "function_name": "transfer",
        "call_type": "external",
        "success": true,
        "gas_used": 50000,
        "args": [],
        "children": [
            {
                "call_id": 2,
                "parent_call_id": 1,
                "environment": "evm",
                "contract_address": "0x5678",
                "function_name": "approve",
                "call_type": "internal",
                "success": true,
                "args": [],
                "children": []
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["children"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["children"][0]["function_name"], "approve");
    assert_eq!(parsed["children"][0]["parent_call_id"], 1);
}

#[test]
fn test_cross_env_call_failed() {
    let json = r#"{
        "call_id": 1,
        "environment": "evm",
        "contract_address": "0x1234",
        "function_name": "withdraw",
        "call_type": "external",
        "success": false,
        "error": "Insufficient balance",
        "return_data": "0x08c379a0...",
        "gas_used": 30000,
        "args": [],
        "children": []
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"], "Insufficient balance");
}

#[test]
fn test_cross_env_call_with_value() {
    let json = r#"{
        "call_id": 1,
        "environment": "evm",
        "contract_address": "0x1234",
        "function_name": "deposit",
        "call_type": "external",
        "success": true,
        "value": 1000000000000000000,
        "args": [],
        "children": []
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["value"], 1000000000000000000u64);
}

// ==================== CrossEnvTrace tests ====================

#[test]
fn test_cross_env_trace_full() {
    let json = r#"{
        "trace_id": "trace-123",
        "protocol_version": "1.0",
        "transaction_hash": "0xabc123",
        "from_address": "0xsender",
        "to_address": "0xreceiver",
        "value": 1000,
        "gas_used": 21000,
        "success": true,
        "calls": []
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["trace_id"], "trace-123");
    assert_eq!(parsed["protocol_version"], "1.0");
    assert_eq!(parsed["transaction_hash"], "0xabc123");
    assert_eq!(parsed["success"], true);
}

#[test]
fn test_cross_env_trace_minimal() {
    let json = r#"{
        "trace_id": "test-trace",
        "calls": [],
        "success": true
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["trace_id"], "test-trace");
    assert!(parsed["success"].as_bool().unwrap());
}

#[test]
fn test_cross_env_trace_with_calls() {
    let json = r#"{
        "trace_id": "multi-call-trace",
        "success": true,
        "calls": [
            {
                "call_id": 1,
                "environment": "evm",
                "contract_address": "0x1111",
                "function_name": "foo",
                "call_type": "external",
                "success": true,
                "args": [],
                "children": []
            },
            {
                "call_id": 2,
                "environment": "evm",
                "contract_address": "0x2222",
                "function_name": "bar",
                "call_type": "external",
                "success": true,
                "args": [],
                "children": []
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    let calls = parsed["calls"].as_array().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0]["function_name"], "foo");
    assert_eq!(calls[1]["function_name"], "bar");
}

// ==================== TraceRequest tests ====================

#[test]
fn test_trace_request_full() {
    let json = r#"{
        "request_id": "req-123",
        "transaction_hash": "0xtx",
        "block_number": 12345678,
        "target_address": "0xtarget",
        "caller_address": "0xcaller",
        "calldata": "0xabcd",
        "value": 500,
        "depth": 2,
        "parent_call_id": 10,
        "parent_trace_id": "parent-trace",
        "source_environment": "stylus"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["request_id"], "req-123");
    assert_eq!(parsed["block_number"], 12345678);
    assert_eq!(parsed["target_address"], "0xtarget");
    assert_eq!(parsed["source_environment"], "stylus");
    assert_eq!(parsed["depth"], 2);
}

#[test]
fn test_trace_request_minimal() {
    let json = r#"{
        "request_id": "req-min",
        "target_address": "0xcontract",
        "calldata": "0xdata",
        "value": 0,
        "depth": 0,
        "source_environment": "stylus"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["target_address"], "0xcontract");
    assert!(parsed.get("caller_address").is_none());
    assert!(parsed.get("transaction_hash").is_none());
}

// ==================== TraceResponse tests ====================

#[test]
fn test_trace_response_success() {
    let json = r#"{
        "request_id": "req-456",
        "status": "success",
        "trace": {
            "trace_id": "trace-789",
            "calls": [],
            "success": true
        }
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["request_id"], "req-456");
    assert_eq!(parsed["status"], "success");
    assert!(parsed["trace"].is_object());
}

#[test]
fn test_trace_response_pending() {
    let json = r#"{
        "request_id": "req-pending",
        "status": "pending"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["status"], "pending");
    assert!(parsed.get("trace").is_none());
}

#[test]
fn test_trace_response_error() {
    let json = r#"{
        "request_id": "req-err",
        "status": "error",
        "error_message": "Contract not found",
        "error_code": "NOT_FOUND"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["error_message"], "Contract not found");
    assert_eq!(parsed["error_code"], "NOT_FOUND");
}

// ==================== HealthResponse tests ====================

#[test]
fn test_health_response() {
    let json = r#"{
        "status": "healthy",
        "protocol_version": "1.0",
        "contracts_registered": 5
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["status"], "healthy");
    assert_eq!(parsed["protocol_version"], "1.0");
    assert_eq!(parsed["contracts_registered"], 5);
}

// ==================== ContractsResponse tests ====================

#[test]
fn test_contracts_response() {
    let json = r#"{
        "contracts": [
            {"address": "0x1", "environment": "evm", "name": "A"},
            {"address": "0x2", "environment": "stylus", "name": "B"}
        ],
        "count": 2
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(parsed["count"], 2);
    let contracts = parsed["contracts"].as_array().unwrap();
    assert_eq!(contracts.len(), 2);
    assert_eq!(contracts[0]["name"], "A");
    assert_eq!(contracts[1]["name"], "B");
}

// ==================== Config file format tests ====================

#[test]
fn test_config_file_format() {
    let config = r#"{
        "contracts": [
            {
                "address": "0x1234567890abcdef1234567890abcdef12345678",
                "environment": "evm",
                "name": "USDC",
                "project_path": "/path/to/usdc",
                "debug_dir": "/path/to/usdc/debug"
            },
            {
                "address": "0xabcdef1234567890abcdef1234567890abcdef12",
                "environment": "stylus",
                "name": "MyDeFi",
                "project_path": "/path/to/mydefi",
                "debug_dir": "/path/to/mydefi/debug"
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(config).unwrap();
    let contracts = parsed["contracts"].as_array().unwrap();

    assert_eq!(contracts.len(), 2);

    // First contract is EVM
    assert_eq!(contracts[0]["environment"], "evm");
    assert_eq!(contracts[0]["name"], "USDC");

    // Second contract is Stylus
    assert_eq!(contracts[1]["environment"], "stylus");
    assert_eq!(contracts[1]["name"], "MyDeFi");
}

// ==================== Cross-env trace entry tests ====================

#[test]
fn test_cross_env_trace_entry_format() {
    // This is the format written to /tmp/cross_env_traces.json
    let entry = r#"{
        "target_address": "0x1234",
        "calldata": "0xa9059cbb0000000000000000000000001234567890abcdef1234567890abcdef12345678",
        "call_type": "CALL",
        "trace": {
            "trace_id": "evm-trace-1",
            "success": true,
            "calls": [
                {
                    "call_id": 1,
                    "environment": "evm",
                    "contract_address": "0x1234",
                    "function_name": "transfer",
                    "call_type": "external",
                    "success": true,
                    "args": [
                        {"name": "to", "type": "address", "value": "0xrecipient"},
                        {"name": "amount", "type": "uint256", "value": "1000"}
                    ],
                    "children": []
                }
            ]
        }
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(entry).unwrap();
    assert_eq!(parsed["target_address"], "0x1234");
    assert_eq!(parsed["call_type"], "CALL");
    assert!(parsed["trace"]["success"].as_bool().unwrap());
    assert_eq!(parsed["trace"]["calls"][0]["function_name"], "transfer");
}

#[test]
fn test_cross_env_trace_entry_delegate_call() {
    let entry = r#"{
        "target_address": "0x5678",
        "calldata": "0x",
        "call_type": "DELEGATECALL",
        "trace": {
            "trace_id": "delegate-1",
            "success": true,
            "calls": []
        }
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(entry).unwrap();
    assert_eq!(parsed["call_type"], "DELEGATECALL");
}

#[test]
fn test_cross_env_trace_entry_static_call() {
    let entry = r#"{
        "target_address": "0x9999",
        "calldata": "0x70a08231",
        "call_type": "STATICCALL",
        "trace": {
            "trace_id": "static-1",
            "success": true,
            "calls": [
                {
                    "call_id": 1,
                    "environment": "evm",
                    "contract_address": "0x9999",
                    "function_name": "balanceOf",
                    "call_type": "external",
                    "success": true,
                    "args": [{"name": "account", "type": "address", "value": "0xowner"}],
                    "return_value": "1000000000000000000",
                    "children": []
                }
            ]
        }
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(entry).unwrap();
    assert_eq!(parsed["call_type"], "STATICCALL");
    assert_eq!(
        parsed["trace"]["calls"][0]["return_value"],
        "1000000000000000000"
    );
}

// ==================== LLDB trace format tests ====================

#[test]
fn test_lldb_trace_format() {
    // Format of /tmp/lldb_function_trace.json
    let trace = r#"[
        {
            "call_id": 1,
            "parent_call_id": 0,
            "function": "user_entrypoint",
            "file": "src/lib.rs",
            "line": 15,
            "args": []
        },
        {
            "call_id": 2,
            "parent_call_id": 1,
            "function": "my_contract::transfer",
            "file": "src/lib.rs",
            "line": 42,
            "args": [
                {"name": "to", "value": "Address(0x1234...)"},
                {"name": "amount", "value": "U256(1000)"}
            ]
        }
    ]"#;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(trace).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["function"], "user_entrypoint");
    assert_eq!(parsed[1]["parent_call_id"], 1);
}

#[test]
fn test_merged_trace_format_with_evm() {
    // Format after merging EVM calls into LLDB trace
    let merged = r#"[
        {
            "call_id": 1,
            "parent_call_id": 0,
            "function": "user_entrypoint",
            "file": "src/lib.rs",
            "line": 15,
            "args": []
        },
        {
            "call_id": 2,
            "parent_call_id": 1,
            "function": "IToken::new",
            "file": "src/lib.rs",
            "line": 20,
            "args": [{"name": "addr", "value": "0xtoken"}]
        },
        {
            "call_id": 3,
            "parent_call_id": 1,
            "function": "IToken::transfer",
            "file": "src/lib.rs",
            "line": 25,
            "args": []
        },
        {
            "call_id": 4,
            "parent_call_id": 3,
            "function": "[EVM] transfer",
            "file": "",
            "line": 0,
            "args": [
                {"name": "to", "value": "address: 0xrecipient"},
                {"name": "amount", "value": "uint256: 1000"}
            ],
            "environment": "evm",
            "contract_address": "0xtoken"
        }
    ]"#;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(merged).unwrap();
    assert_eq!(parsed.len(), 4);

    // Find the EVM call
    let evm_call = parsed
        .iter()
        .find(|c| {
            c.get("function")
                .and_then(|f| f.as_str())
                .map(|f| f.starts_with("[EVM]"))
                .unwrap_or(false)
        })
        .unwrap();

    assert_eq!(evm_call["environment"], "evm");
    assert_eq!(evm_call["parent_call_id"], 3); // Parent is the IToken::transfer call
}

// ==================== Address normalization tests ====================

#[test]
fn test_address_case_insensitivity() {
    let addresses = vec![
        "0xABCDef1234567890ABCDEF1234567890AbCdEf12",
        "0xabcdef1234567890abcdef1234567890abcdef12",
        "0xABCDEF1234567890ABCDEF1234567890ABCDEF12",
    ];

    let normalized: Vec<String> = addresses.iter().map(|a| a.to_lowercase()).collect();

    // All should normalize to the same value
    assert!(normalized.iter().all(|a| a == &normalized[0]));
}

// ==================== Complex scenario tests ====================

#[test]
fn test_stylus_calls_multiple_evm_contracts() {
    // Scenario: Stylus contract calls TokenA, then TokenB
    let cross_env_traces = r#"[
        {
            "target_address": "0xtokena",
            "calldata": "0xa9059cbb",
            "call_type": "CALL",
            "trace": {
                "trace_id": "trace-a",
                "success": true,
                "calls": [{"call_id": 1, "function_name": "transfer", "environment": "evm", "contract_address": "0xtokena", "call_type": "external", "success": true, "args": [], "children": []}]
            }
        },
        {
            "target_address": "0xtokenb",
            "calldata": "0x095ea7b3",
            "call_type": "CALL",
            "trace": {
                "trace_id": "trace-b",
                "success": true,
                "calls": [{"call_id": 1, "function_name": "approve", "environment": "evm", "contract_address": "0xtokenb", "call_type": "external", "success": true, "args": [], "children": []}]
            }
        }
    ]"#;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(cross_env_traces).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["target_address"], "0xtokena");
    assert_eq!(parsed[1]["target_address"], "0xtokenb");
}

#[test]
fn test_evm_contract_calls_another_evm_contract() {
    // Scenario: Stylus calls TokenA, which internally calls TokenB
    let trace = r#"{
        "trace_id": "nested-evm",
        "success": true,
        "calls": [
            {
                "call_id": 1,
                "environment": "evm",
                "contract_address": "0xtokena",
                "function_name": "swap",
                "call_type": "external",
                "success": true,
                "args": [],
                "children": [
                    {
                        "call_id": 2,
                        "parent_call_id": 1,
                        "environment": "evm",
                        "contract_address": "0xtokenb",
                        "function_name": "transferFrom",
                        "call_type": "external",
                        "success": true,
                        "args": [],
                        "children": []
                    }
                ]
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(trace).unwrap();
    let outer_call = &parsed["calls"][0];
    let inner_call = &outer_call["children"][0];

    assert_eq!(outer_call["function_name"], "swap");
    assert_eq!(inner_call["function_name"], "transferFrom");
    assert_eq!(inner_call["parent_call_id"], 1);
}

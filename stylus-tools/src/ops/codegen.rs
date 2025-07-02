// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{collections::HashMap, fmt::Write, fs, io::BufReader, path::Path};

use alloy::json_abi::{Function, JsonAbi, StateMutability};
use eyre::{bail, Result};
use serde_json::Value;
use tiny_keccak::{Hasher, Keccak};

fn c_bytearray_initializer(val: &[u8]) -> String {
    let inner: Vec<_> = val.iter().map(|byte| format!("0x{byte:02x}")).collect();
    format!("{{{}}}", inner.join(", "))
}

pub fn c_gen(in_path: impl AsRef<Path>, out_path: impl AsRef<Path>) -> Result<()> {
    let f = fs::File::open(&in_path)?;

    let input: Value = serde_json::from_reader(BufReader::new(f))?;

    let Some(input_contracts) = input["contracts"].as_object() else {
        bail!(
            "did not find top-level contracts object in {}",
            in_path.as_ref().to_string_lossy()
        )
    };

    let mut pathbuf = std::path::PathBuf::new();
    pathbuf.push(out_path);

    for (solidity_file_name, solidity_file_out) in input_contracts {
        let debug_path = vec![solidity_file_name.as_str()];
        let Some(contracts) = solidity_file_out.as_object() else {
            println!("skipping output for {:?} not an object..", &debug_path);
            continue;
        };
        pathbuf.push(solidity_file_name);
        fs::create_dir_all(&pathbuf)?;

        for (contract_name, contract_val) in contracts {
            let mut debug_path = debug_path.clone();
            debug_path.push(contract_name);

            let Some(properties) = contract_val.as_object() else {
                println!("skipping output for {:?} not an object..", &debug_path);
                continue;
            };

            let mut methods: HashMap<String, Vec<Function>> = HashMap::default();

            if let Some(raw) = properties.get("abi") {
                // Sadly, JsonAbi = serde_json::from_value is not supported.
                // Tonight, we hack!
                let abi_json = serde_json::to_string(raw)?;
                let abi: JsonAbi = serde_json::from_str(&abi_json)?;
                for function in abi.functions() {
                    let name = function.name.clone();
                    methods.entry(name).or_default().push(function.clone());
                }
            } else {
                println!("skipping abi for {:?}: not found", &debug_path);
            }

            let mut header = String::default();
            let mut router = String::default();

            for (simple_name, mut overloads) in methods {
                overloads.sort_by_key(|a| a.signature());

                for (index, overload) in overloads.iter().enumerate() {
                    let c_name = match index {
                        0 => simple_name.clone(),
                        x => format!("{simple_name}_{x}"),
                    };
                    let selector = u32::from_be_bytes(*overload.selector());

                    let (hdr_params, call_params, payable) = match overload.state_mutability {
                        StateMutability::Pure => {
                            ("(uint8_t *input, size_t len)", "(input, len)", false)
                        }
                        StateMutability::View => (
                            "(const void *storage, uint8_t *input, size_t len)",
                            "(NULL, input, len)",
                            false,
                        ),
                        StateMutability::NonPayable => (
                            "(void *storage, uint8_t *input, size_t len)",
                            "(NULL, input, len)",
                            false,
                        ),
                        StateMutability::Payable => (
                            "(void *storage, uint8_t *input, size_t len, bebi32 value)",
                            "(NULL, input, len, value)",
                            true,
                        ),
                    };

                    let sig = &overload.signature();
                    writeln!(
                        header,
                        "#define SELECTOR_{c_name} 0x{selector:08x} // {sig}"
                    )?;
                    writeln!(header, "ArbResult {c_name}{hdr_params}; // {sig}")?;

                    writeln!(router, "    if (selector==SELECTOR_{c_name}) {{")?;
                    if !payable {
                        writeln!(router, "        if (!bebi32_is_zero(value)) revert();")?;
                    }
                    writeln!(router, "        return {c_name}{call_params};\n    }}")?;
                }
            }

            if !header.is_empty() {
                header.push('\n');
            }
            debug_path.push("storageLayout");

            if let Some(Value::Object(layout_vals)) = properties.get("storageLayout") {
                debug_path.push("storage");

                if let Some(Value::Array(storage_arr)) = layout_vals.get("storage") {
                    for storage_val in storage_arr {
                        let Some(storage_obj) = storage_val.as_object() else {
                            println!("skipping output inside {debug_path:?}: not an object..");
                            continue;
                        };
                        let Some(Value::String(label)) = storage_obj.get("label") else {
                            println!("skipping output inside {debug_path:?}: no label..");
                            continue;
                        };
                        let Some(Value::String(slot)) = storage_obj.get("slot") else {
                            println!("skipping output inside {debug_path:?}: no slot..");
                            continue;
                        };
                        let Ok(slot) = slot.parse::<u64>() else {
                            println!("skipping output inside {debug_path:?}: slot not u64..");
                            continue;
                        };
                        let Some(Value::String(val_type)) = storage_obj.get("type") else {
                            println!("skipping output inside {debug_path:?}: no type..");
                            continue;
                        };
                        let Some(Value::Number(read_offset)) = storage_obj.get("offset") else {
                            println!("skipping output inside {debug_path:?}: no offset..");
                            continue;
                        };
                        let offset = match read_offset.as_i64() {
                            None => {
                                println!(
                                    "skipping output inside {debug_path:?}: unexpected offset..",
                                );
                                continue;
                            }
                            Some(num) => {
                                if !(0..=32).contains(&num) {
                                    println!(
                                        "skipping output inside {debug_path:?}: unexpected offset..",
                                    );
                                    continue;
                                };
                                32 - num
                            }
                        };
                        let mut slot_buf = vec![0u8; 32 - 8];
                        slot_buf.extend(slot.to_be_bytes());

                        writeln!(
                            header,
                            "#define STORAGE_SLOT_{label} {} // {val_type}",
                            c_bytearray_initializer(&slot_buf),
                        )?;
                        if val_type.starts_with("t_array(") {
                            if val_type.ends_with(")dyn_storage") {
                                let mut keccak = Keccak::v256();
                                keccak.update(&slot_buf);
                                keccak.finalize(&mut slot_buf);
                                writeln!(
                                    header,
                                    "#define STORAGE_BASE_{label} {} // {val_type}",
                                    c_bytearray_initializer(&slot_buf),
                                )?;
                            }
                        } else if !val_type.starts_with("t_mapping") {
                            writeln!(
                                header,
                                "#define STORAGE_END_OFFSET_{label} {offset} // {val_type}",
                            )?;
                        }
                    }
                } else {
                    println!("skipping output for {debug_path:?}: not an array..");
                }
                debug_path.pop();
            } else {
                println!("skipping output for {:?}: not an object..", &debug_path);
            }
            debug_path.pop();
            if !header.is_empty() {
                let mut unique_identifier = String::from("__");
                unique_identifier += &solidity_file_name.to_uppercase();
                unique_identifier += "_";
                unique_identifier += &contract_name.to_uppercase();
                unique_identifier += "_";

                let contents = format!(
                    r#" // autogenerated by cargo-stylus
#ifndef {uniq}
#define {uniq}

#include <stylus_types.h>
#include <bebi.h>

#ifdef __cplusplus
extern "C" {{
#endif

ArbResult default_func(void *storage, uint8_t *input, size_t len, bebi32 value);

{body}

#ifdef __cplusplus
}}
#endif

#endif // {uniq}
"#,
                    uniq = unique_identifier,
                    body = header
                );

                let filename: String = contract_name.into();
                pathbuf.push(filename + ".h");
                fs::write(&pathbuf, &contents)?;
                pathbuf.pop();
            }
            if !router.is_empty() {
                let contents = format!(
                    r#" // autogenerated by cargo-stylus

#include "{contract}.h"
#include <stylus_types.h>
#include <stylus_entry.h>
#include <bebi.h>


ArbResult {contract}_entry(uint8_t *input, size_t len) {{
    bebi32 value;
    msg_value(value);
    if (len < 4) {{
        return default_func(NULL, input, len, value);
    }}
    uint32_t selector = bebi_get_u32(input, 0);
    input +=4;
    len -=4;
{body}
    input -=4;
    len +=4;
    return default_func(NULL, input, len, value);
}}

ENTRYPOINT({contract}_entry)
"#,
                    contract = contract_name,
                    body = router
                );

                let filename: String = contract_name.into();
                pathbuf.push(filename + "_main.c");
                fs::write(&pathbuf, &contents)?;
                pathbuf.pop();
            }
        }
        pathbuf.pop();
    }
    Ok(())
}

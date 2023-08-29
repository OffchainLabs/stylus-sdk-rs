#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::stylus_proc::entrypoint;

use json_lib::parse_json;

#[entrypoint]
fn user_main(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
    parse_json(input)
}

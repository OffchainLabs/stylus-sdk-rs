#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::stylus_proc::entrypoint;

use json_lib::json_main;

#[entrypoint]
fn user_main(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
    json_main(input)
}

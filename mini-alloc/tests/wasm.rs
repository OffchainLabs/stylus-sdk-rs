#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use wasm_bindgen_test::*;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[wasm_bindgen_test]
fn it_works() {
    let p1 = Vec::<u8>::with_capacity(700);
    assert!(p1.as_ptr() as usize > 20000);
    let p2 = Vec::<u8>::with_capacity(65536);
    assert_eq!(p1.as_ptr() as usize + 700, p2.as_ptr() as usize);
}

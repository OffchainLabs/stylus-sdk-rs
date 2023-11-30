#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[wasm_bindgen_test]
#[cfg(target_arch = "wasm32")]
fn vec_test() {
    let p1 = Vec::<u8>::with_capacity(700);
    let p2 = Vec::<u8>::with_capacity(65536);
    assert_eq!(p1.as_ptr() as usize + 700, p2.as_ptr() as usize);
    let p3 = Vec::<u8>::with_capacity(1);
    assert_eq!(p2.as_ptr() as usize + 65536, p3.as_ptr() as usize);
}

#[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
#[test]
fn vec_test() {
    let p1 = Vec::<u8>::with_capacity(700);
    let p2 = Vec::<u8>::with_capacity(65536);
    assert_eq!(p1.as_ptr() as usize, p2.as_ptr() as usize + 65536);
    let p3 = Vec::<u8>::with_capacity(1);
    assert_eq!(p2.as_ptr() as usize, p3.as_ptr() as usize + 1);
}

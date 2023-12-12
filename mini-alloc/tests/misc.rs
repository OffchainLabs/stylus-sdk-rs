// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

#![no_std]

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn vec_test() {
    use alloc::vec::Vec;

    let p1 = Vec::<u8>::with_capacity(700);
    let p2 = Vec::<u8>::with_capacity(65536);
    let p3 = Vec::<u8>::with_capacity(700000);
    let p4 = Vec::<u32>::with_capacity(1);
    let p5 = Vec::<u8>::with_capacity(1);
    let p6 = Vec::<u16>::with_capacity(1);
    assert_eq!(p1.as_ptr() as usize + 700, p2.as_ptr() as usize);
    assert_eq!(p2.as_ptr() as usize + 65536, p3.as_ptr() as usize);
    assert!((p4.as_ptr() as usize) < p3.as_ptr() as usize + 700004);
    assert!((p4.as_ptr() as usize) >= p3.as_ptr() as usize + 700000);
    assert_eq!(p4.as_ptr() as usize & 3, 0);
    assert_eq!(p4.as_ptr() as usize + 4, p5.as_ptr() as usize);
    assert_eq!(p5.as_ptr() as usize + 2, p6.as_ptr() as usize);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn vec_test_loop() {
    use alloc::vec::Vec;

    let mut size = 1usize;
    let mut p = Vec::<u8>::with_capacity(size);
    for _ in 0 .. 22 {
        let new_size = size*2;
        let new_p = Vec::<u8>::with_capacity(new_size);
        assert_eq!(p.as_ptr() as usize + size, new_p.as_ptr() as usize);
        size = new_size;
        p = new_p;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
#[should_panic]
fn vec_test_overallocate() {
    use alloc::vec::Vec;

    let _ = Vec::<u8>::with_capacity(0xFFFFFFFF);
}

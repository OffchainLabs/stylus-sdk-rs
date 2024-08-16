// Copyright 2023-2024, Offchain Labs, Inc.
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
    for _ in 0..22 {
        let new_size = size * 2;
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn edge_cases() {
    use alloc::alloc::{alloc, Layout};
    use core::arch::wasm32;
    use mini_alloc::MiniAlloc;

    const PAGE_SIZE: usize = MiniAlloc::PAGE_SIZE;
    const PAGE_LIMIT: usize = 65536;

    fn size() -> usize {
        wasm32::memory_size(0) as usize
    }

    fn size_bytes() -> usize {
        size() * PAGE_SIZE
    }

    fn next(size: usize) -> usize {
        let align = 1;
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { alloc(layout) as usize }
    }

    assert_eq!(size(), 1);

    // check that zero-allocs don't bump
    let start = next(0);
    assert_eq!(start, next(0));
    assert_eq!(start / PAGE_SIZE, 0);
    assert_eq!(size(), 1);

    // fill the rest of the page
    let rest = size_bytes() - start;
    let end = next(rest);
    assert_eq!(end / PAGE_SIZE, 0);
    assert_eq!(end, start);
    assert_eq!(size(), 1);

    // allocate a second page
    let first = next(1);
    assert_eq!(first / PAGE_SIZE, 1);
    assert_eq!(first, PAGE_SIZE);
    assert_eq!(size(), 2);

    // fill the rest of the second page
    let rest = size_bytes() - (first + 1);
    let end = next(rest);
    assert_eq!(end, first + 1);
    assert_eq!(size(), 2);

    // jump 4 pages
    let jump = next(4 * PAGE_SIZE);
    assert_eq!(jump, 2 * PAGE_SIZE);
    assert_eq!(size(), 6);

    // allocate many pages
    let mut rng: usize = 0;
    while size() < PAGE_LIMIT / 2 {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);

        let rest = usize::MAX - next(0) + 1;
        let bytes = match rng % 4 {
            0 => rng % 1024,
            1 => rng % PAGE_SIZE,
            2 => next(size_bytes() - next(0)), // rest of page
            _ => rng % (PAGE_SIZE * 200),
        };

        let offset = next(bytes.min(rest));

        if offset == size_bytes() {
            assert_eq!(bytes, 0);
        } else {
            assert!(offset < size_bytes());
        }
    }

    // TODO: test allocating all 4GB
}

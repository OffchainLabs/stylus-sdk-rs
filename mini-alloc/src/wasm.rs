// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::wasm32,
    num::NonZeroUsize as NonZero,
};

pub struct MiniAlloc;

/// This is not a valid implementation of [`Sync`] but is ok in single-threaded WASM.
unsafe impl Sync for MiniAlloc {}

impl MiniAlloc {
    pub const INIT: Self = MiniAlloc;
}

unsafe impl GlobalAlloc for MiniAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc_impl(layout).unwrap_or(core::ptr::null_mut())
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

extern "C" {
    /// This symbol is created by the LLVM linker.
    static __heap_base: u8;
}

/// The WASM page size, or 2^16 bytes.
const PAGE_SIZE: usize = 1 << 16;

/// Represents the allocator's bump offset and boundary
static mut STATE: Option<(NonZero, usize)> = None;

pub unsafe fn alloc_impl(layout: Layout) -> Option<*mut u8> {
    let (offset, bound) = STATE.get_or_insert_with(|| {
        let offset = NonZero::new_unchecked(&__heap_base as *const _ as _);
        let bound = PAGE_SIZE * wasm32::memory_size(0) - 1; // last offset inbounds
        (offset, bound)
    });

    let aligned = make_aligned(offset.get(), layout.align())?;
    let next_offset = aligned.checked_add(layout.size())?;

    if next_offset - 1 > *bound {
        let pages = 1 + (next_offset - *bound - 2) / PAGE_SIZE;
        match wasm32::memory_grow(0, pages) {
            usize::MAX => return None,
            _ => *bound += PAGE_SIZE * pages,
        }
    }
    *offset = NonZero::new_unchecked(next_offset);
    Some(aligned as *mut _)
}

/// Returns `value` rounded up to the next multiple of `align`.
/// Note: `align` must be a power of two, which is guaranteed by [`Layout::align`].
#[inline(always)]
fn make_aligned(value: usize, align: usize) -> Option<usize> {
    let x = value.checked_add(align - 1)?;
    Some(x & align.wrapping_neg())
}

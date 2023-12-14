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

    /// The WASM page size, or 2^16 bytes.
    pub const PAGE_SIZE: usize = 1 << 16;
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

/// Represents the negation of the allocator's bump offset and boundary
///
/// We store the negation because we can align the negative offset in fewer
/// instructions than the positive offset.
static mut STATE: Option<(NonZero, usize)> = None;

unsafe fn alloc_impl(layout: Layout) -> Option<*mut u8> {
    let (neg_offset, neg_bound) = STATE.get_or_insert_with(|| {
        let heap_base = &__heap_base as *const u8 as usize;
        let bound = MiniAlloc::PAGE_SIZE * wasm32::memory_size(0) - 1;
        (
            NonZero::new_unchecked(heap_base.wrapping_neg()),
            bound.wrapping_neg(),
        )
    });

    let neg_aligned = make_aligned(neg_offset.get(), layout.align());
    let next_neg_offset = neg_aligned.checked_sub(layout.size())?;
    let bytes_needed = neg_bound.saturating_sub(next_neg_offset + 1);
    if bytes_needed != 0 {
        let pages_needed = 1 + (bytes_needed - 1) / MiniAlloc::PAGE_SIZE;
        if wasm32::memory_grow(0, pages_needed) == usize::MAX {
            return None;
        }
        *neg_bound -= MiniAlloc::PAGE_SIZE * pages_needed;
    }
    *neg_offset = NonZero::new_unchecked(next_neg_offset);
    Some(neg_aligned.wrapping_neg() as *mut u8)
}

/// Returns `value` rounded down to the next multiple of `align`.
/// Note: `align` must be a power of two, which is guaranteed by [`Layout::align`].
#[inline(always)]
fn make_aligned(value: usize, align: usize) -> usize {
    value & align.wrapping_neg()
}

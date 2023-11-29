#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;

const PAGE_SIZE: usize = 0x10000;
const PAGE_MASK: usize = !0xFFFF;

fn heap_base() -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    todo!();

    #[cfg(target_arch = "wasm32")]
    {
        extern "C" {
            // This symbol is created by the LLVM linker.
            static __heap_base: u8;
        }

        unsafe { &__heap_base as *const u8 as usize }
    }
}

#[derive(Default)]
pub struct MiniAlloc {
    start: Cell<usize>,
}

unsafe impl Sync for MiniAlloc {}

impl MiniAlloc {
    pub const INIT: Self = MiniAlloc {
        start: Cell::new(0),
    };
}

fn round_up_to_alignment(val: usize, align: usize) -> usize {
    (val + align - 1) & (-(align as isize) as usize)
}

fn size() -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    todo!();

    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::memory_size(0)
}

fn grow(pages: usize) -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = pages;
        todo!();
    }

    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::memory_grow(0, pages)
}

unsafe impl GlobalAlloc for MiniAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.start.get() == 0 {
            self.start.set(heap_base())
        }

        let memory_bytes = size() * PAGE_SIZE;
        let mut this_start = round_up_to_alignment(self.start.get(), layout.align());
        let mut next_start = this_start + layout.size();

        if cfg!(not(all())) {
            // We expect memory_grow will never be called by anyone else, so
            // this check is unnecessary.
            if memory_bytes.saturating_sub(this_start) >= PAGE_SIZE
                && (next_start - 1) & PAGE_MASK > this_start
            {
                // Someone else has grown memory, and the requested allocation
                // won't fit on our current page.
                this_start = round_up_to_alignment(memory_bytes, layout.align());
                next_start = this_start + layout.size();
            }
        }

        let needed_bytes = next_start.saturating_sub(memory_bytes);
        let needed_pages = (needed_bytes + PAGE_SIZE - 1) / PAGE_SIZE;
        grow(needed_pages);
        self.start.set(next_start);
        this_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

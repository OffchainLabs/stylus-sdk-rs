use core::alloc::Layout;
use core::arch::wasm32::{memory_grow, memory_size};

const PAGE_SIZE: usize = 0x10000;

pub struct AllocImpl {
    pointer: usize,
}

pub static mut ALLOC_IMPL: AllocImpl = AllocImpl {
    pointer: 0,
};

/// This is not a valid implementation of `Sync`.
unsafe impl Sync for AllocImpl {}

impl AllocImpl {
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        self.maybe_initialize_heap();
        let this_pointer = match round_up_to_alignment(self.pointer, layout.align()) {
            Ok(x) => x,
            Err(()) => return core::ptr::null_mut(),
        };
        let next_pointer = match this_pointer.checked_add(layout.size()) {
            Some(x) => x,
            None => return core::ptr::null_mut(),
        };
        let needed_bytes = next_pointer.saturating_sub(memory_size(0));
        let needed_pages = (PAGE_SIZE - 1 + needed_bytes) / PAGE_SIZE;
        memory_grow(0, needed_pages);
        self.pointer = next_pointer;
        this_pointer as *mut u8
    }


    fn maybe_initialize_heap(&mut self) {
        if self.pointer != 0 {
            return;
        }

        extern "C" {
            // This symbol is created by the LLVM linker.
            static __heap_base: u8;
        }

        self.pointer = unsafe { &__heap_base as *const u8 as usize };
    }
}

/// `align` must be a power of two.
const fn round_up_to_alignment(val: usize, align: usize) -> Result<usize, ()> {
    match val.checked_add(align - 1) {
        Some(x) => Ok(x & (-(align as isize) as usize)),
        None => Err(()),
    }
}

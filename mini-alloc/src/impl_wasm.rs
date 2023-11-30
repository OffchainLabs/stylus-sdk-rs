use core::alloc::Layout;
use core::arch::wasm32::{memory_grow, memory_size};

const PAGE_SIZE: usize = 0x10000;

static mut POINTER: usize = 0;

pub fn alloc(layout: Layout) -> *mut u8 {
    maybe_initialize_heap();
    let this_pointer = match round_up_to_alignment(unsafe { POINTER }, layout.align()) {
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
    unsafe {
        POINTER = next_pointer;
    }
    this_pointer as *mut u8
}

fn maybe_initialize_heap() {
    if unsafe { POINTER } != 0 {
        return;
    }

    extern "C" {
        // This symbol is created by the LLVM linker.
        static __heap_base: u8;
    }

    unsafe {
        POINTER = &__heap_base as *const u8 as usize;
    }
}

/// `align` must be a power of two.
const fn round_up_to_alignment(val: usize, align: usize) -> Result<usize, ()> {
    match val.checked_add(align - 1) {
        Some(x) => Ok(x & (-(align as isize) as usize)),
        None => Err(()),
    }
}

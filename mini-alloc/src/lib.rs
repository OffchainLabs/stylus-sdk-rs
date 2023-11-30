#![no_std]

use core::alloc::{GlobalAlloc, Layout};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod impl_wasm;
        use impl_wasm as impl_mod;
    } else if #[cfg(any(unix, windows))] {
        mod impl_unix_windows;
        use impl_unix_windows as impl_mod;
    }
}

pub struct MiniAlloc(());

/// This is not a valid implementation of `Sync`.
unsafe impl Sync for MiniAlloc {}

impl MiniAlloc {
    pub const INIT: Self = MiniAlloc(());
}

unsafe impl GlobalAlloc for MiniAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        impl_mod::alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

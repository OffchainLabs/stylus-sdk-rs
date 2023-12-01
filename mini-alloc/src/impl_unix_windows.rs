use core::alloc::Layout;

const PAGE_SIZE: usize = 0x10000;
const MAX_PAGES: usize = 1024;

pub struct AllocImpl {
    pointer: usize,
    heap_first: usize,
    pages_committed: usize,
}

pub static mut ALLOC_IMPL: AllocImpl = AllocImpl {
    pointer: 0,
    heap_first: 0,
    pages_committed: 0,
};

/// This is not a valid implementation of `Sync`.
unsafe impl Sync for AllocImpl {}

impl AllocImpl {
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if self.maybe_initialize_heap().is_err() {
            return core::ptr::null_mut();
        }
        let maybe_pointer = self.pointer.saturating_sub(layout.size());
        // We grow the heap down, because round_down_to_alignment is a little
        // faster than round_up_to_alignment and we don't have to do as many
        // overflow checks.
        let real_pointer = round_down_to_alignment(maybe_pointer, layout.align());
        let needed_bytes = self.heap_first.saturating_sub(real_pointer);
        let needed_pages = (PAGE_SIZE - 1 + needed_bytes) / PAGE_SIZE;
        if self.grow(needed_pages).is_err() {
            return core::ptr::null_mut();
        }
        self.pointer = real_pointer;
        real_pointer as *mut u8
    }

    fn maybe_initialize_heap(&mut self) -> Result<(), ()> {
        if self.heap_first != 0 {
            return Ok(());
        }
        let addr = impls::mem_reserve()?;
        self.heap_first = addr;
        self.pointer = addr;
        Ok(())
    }

    fn grow(&mut self, pages: usize) -> Result<(), ()> {
        if pages == 0 {
            return Ok(());
        }
        let new_committed = self.pages_committed + pages;
        if new_committed > MAX_PAGES {
            return Err(());
        }
        let bytes = pages * PAGE_SIZE;
        let new_heap_first = self.heap_first - bytes;
        impls::mem_commit(new_heap_first, bytes)?;
        self.heap_first = new_heap_first;
        self.pages_committed = new_committed;
        Ok(())
    }
}

/// `align` must be a power of two.
const fn round_down_to_alignment(val: usize, align: usize) -> usize {
    val & (-(align as isize) as usize)
}

#[cfg(unix)]
mod impls {
    use super::*;

    pub fn mem_reserve() -> Result<usize, ()> {
        let len = PAGE_SIZE * MAX_PAGES;
        let ret = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                len,
                libc::PROT_NONE,
                libc::MAP_ANON | libc::MAP_PRIVATE,
                -1,
                0,
            )
        } as usize;
        if ret == usize::MAX {
            Err(())
        } else {
            Ok(ret + len)
        }
    }

    pub fn mem_commit(address: usize, len: usize) -> Result<(), ()> {
        let ret = unsafe {
            libc::mprotect(
                address as *mut libc::c_void,
                len,
                libc::PROT_READ | libc::PROT_WRITE,
            )
        };
        if ret != 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
mod impls {
    use super::*;

    pub fn mem_reserve() -> Result<usize, ()> {
        let len = PAGE_SIZE * MAX_PAGES;
        let ret = unsafe {
            winapi::um::memoryapi::VirtualAlloc(
                winapi::shared::ntdef::NULL,
                len,
                winapi::um::winnt::MEM_RESERVE,
                winapi::um::winnt::PAGE_READWRITE,
            )
        } as usize;
        if ret == 0 {
            Err(())
        } else {
            Ok(ret + len)
        }
    }

    pub fn mem_commit(address: usize, len: usize) -> Result<(), ()> {
        let ret = unsafe {
            winapi::um::memoryapi::VirtualAlloc(
                address as winapi::shared::minwindef::LPVOID,
                len,
                winapi::um::winnt::MEM_COMMIT,
                winapi::um::winnt::PAGE_READWRITE,
            )
        } as usize;
        if ret == 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

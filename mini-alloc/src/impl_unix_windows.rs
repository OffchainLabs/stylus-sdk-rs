use core::alloc::Layout;

const PAGE_SIZE: usize = 0x10000;
const MAX_PAGES: usize = 1024;

static mut POINTER: usize = 0;
static mut HEAP_FIRST: usize = 0;
static mut PAGES_COMMITTED: usize = 0;

pub fn alloc(layout: Layout) -> *mut u8 {
    if maybe_initialize_heap().is_err() {
        return core::ptr::null_mut();
    }
    let maybe_pointer = unsafe { POINTER }.saturating_sub(layout.size());
    if maybe_pointer == 0 {
        return core::ptr::null_mut();
    }

    // We grow the heap down, because round_down_to_alignment is a little faster
    // than round_up_to_alignment.
    let real_pointer = round_down_to_alignment(maybe_pointer, layout.align());
    let needed_bytes = unsafe { HEAP_FIRST }.saturating_sub(real_pointer);
    let needed_pages = (PAGE_SIZE - 1 + needed_bytes) / PAGE_SIZE;
    if grow(needed_pages).is_err() {
        return core::ptr::null_mut();
    }
    unsafe {
        POINTER = real_pointer;
    }
    real_pointer as *mut u8
}

fn maybe_initialize_heap() -> Result<(), ()> {
    if unsafe { HEAP_FIRST } != 0 {
        return Ok(());
    }
    let addr = impls::mem_reserve()?;
    unsafe {
        HEAP_FIRST = addr;
        POINTER = addr;
    }
    Ok(())
}

fn grow(pages: usize) -> Result<(), ()> {
    if pages == 0 {
        return Ok(());
    }
    let new_committed = unsafe { PAGES_COMMITTED } + pages;
    if new_committed > MAX_PAGES {
        return Err(());
    }
    let bytes = pages * PAGE_SIZE;
    let new_heap_first = unsafe { HEAP_FIRST } - bytes;
    impls::mem_commit(new_heap_first, bytes)?;
    unsafe {
        HEAP_FIRST = new_heap_first;
        PAGES_COMMITTED = new_committed;
    }
    Ok(())
}

/// `align` must be a power of two.
const fn round_down_to_alignment(val: usize, align: usize) -> usize {
    val & !(align - 1)
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
                libc::PROT_READ | libc::PROT_WRITE
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
                winapi::um::winnt::PAGE_READWRITE
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
                winapi::um::winnt::PAGE_READWRITE
            )
        } as usize;
        if ret == 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

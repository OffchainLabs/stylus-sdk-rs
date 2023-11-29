# mini-alloc

`mini-alloc` is a very small bump allocator intended to be used as the global
allocator for Rust programs targeting WASM. It never deallocates memory -- that
is, `dealloc` does nothing. It's suitable for cases where binary size is at a
premium and it's acceptable to leak all allocations.

Use it like this:

```rust
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;
```

`core::arch::wasm32::memory_grow` must never be called by any code outside this crate.

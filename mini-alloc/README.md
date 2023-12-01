# mini-alloc

`mini-alloc` is a very small bump allocator intended to be used as the global
Rust allocator. It never deallocates memory -- that is, `dealloc` does nothing.
It's suitable for cases where binary size is at a premium and it's acceptable
to leak all allocations.

One other major limitation: this crate is not thread safe! `MiniAlloc`
implements `Sync` because that is a required of a global allocator, but this is
not a valid implementation of `Sync`, and it must only be used from a single
thread.

Use it like this:

```rust
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc;
```

On wasm, `core::arch::wasm32::memory_grow` must never be called by any code
outside this crate.

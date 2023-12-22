# mini-alloc

`mini-alloc` is a very small bump allocator for wasm32 intended to be used as
the global Rust allocator. It never deallocates memory -- that is, `dealloc`
does nothing.  It's suitable for cases where binary size is at a premium and
it's acceptable to leak all allocations.

One other major limitation: this crate is not thread safe! `MiniAlloc`
implements `Sync` because that is a required of a global allocator, but this is
not a valid implementation of `Sync`, and it must only be used from a single
thread.

Also, `core::arch::wasm32::memory_grow` must never be called by any code outside
this crate.

On targets other than wasm32, `MiniAlloc` simply forwards to the allocator from
another crate, `wee_alloc::WeeAlloc`.

`mini-alloc` uses less ink on 
[Stylus](https://github.com/OffchainLabs/stylus-sdk-rs) compared to other
allocators. When running the `edge_cases` test in this crate on Stylus, here are
ink costs (minus the cost of memory expansion) when using `MiniAlloc` vs
`WeeAlloc` and Rust's default allocator.


|               | MiniAlloc    | WeeAlloc     | Default      |
| ------------- | ------------ | ------------ | ------------ |
| alloc         | 3324474      | 7207816      | 5163328      |
| alloc_zeroed  | 3288777      | 954023099920 | 484822031511 |

`alloc` means `edge_cases` was run as it appears in this crate. `alloc_zeroed`
means the calls to `alloc` were replaced with calls to `alloc_zeroed`.  We can
achieve substantial savings in this case because newly expanded memory in WASM
is already zeroed.

Use `MiniAlloc` like this:

```rust
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;
```

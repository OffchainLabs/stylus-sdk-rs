# mini-alloc

`mini-alloc` is a small and performant allocator optimized for `wasm32` targets like [Arbitrum Stylus][Stylus]. You can use it in your program as follows.
```rust
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;
```

## Benchmarks

`mini-alloc` implements a minimal bump allocator strategy. It never deallocates memory -- that is, `dealloc` does nothing. It's suitable for cases where binary size is at a premium and it's acceptable to leak all allocations. The simplicity of this model makes it very efficient, as seen in the following benchmarks.

|              | `MiniAlloc` | [`WeeAlloc`][WeeAlloc] | Std Library    |
|--------------|-------------|------------------------|----------------|
| alloc        | 333 gas     | 721 gas                | 516 gas        |
| alloc_zeroed | 329 gas     | 95 million gas         | 48 million gas |

The benchmarks compare the performance of this crate's `edge_cases` test in the [Stylus VM][StylusVM]. Normal allocations are over **2x** cheaper than when using [`WeeAlloc`][WeeAlloc], a common WASM alternative that this crate defaults to when built for non-WASM targets.

Replacing each instance of `alloc` in the test with `alloc_zeroed` reveals an over **99%** improvement for zero-filled allocations. Unlike [`WeeAlloc`][WeeAlloc] and the standard library, `MiniAlloc` takes advantage of the fact that WASM pages are zero-filled at initialization, and uses fewer of them due to the layout of Rust's memory.

In the above tests we disable memory expansion costs, which unfairly penelize `WeeAlloc` and the standard library due to their increased resource consumption.

## Notice

`MiniAlloc` should not be used in `wasm32` environments that enable the multithreading proposal. Although `MiniAlloc` implements `Sync` since Rust requires it for the global allocator, this crate should not be used in this way. This should not be a concern in [`Stylus`][Stylus].

Also, `core::arch::wasm32::memory_grow` must never be called by any code outside this crate.

## License

&copy; 2023-2024 Offchain Labs, Inc.

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([licenses/Apache-2.0](../licenses/Apache-2.0))
- [MIT license](https://opensource.org/licenses/MIT) ([licenses/MIT](../licenses/MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

[Stylus]: https://github.com/OffchainLabs/stylus-sdk-rs
[StylusVM]: https://github.com/OffchainLabs/stylus
[WeeAlloc]: https://github.com/rustwasm/wee_alloc

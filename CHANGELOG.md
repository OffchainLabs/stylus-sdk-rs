# Changelog

These crates follow [semver](https://semver.org).

## [0.8.3](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.8.3) - 2025-03-14

### Fixed

- Fix stylus SDK dependencies in wasm32
- Do not require contract crate to define stylus-test feature

## [0.8.2](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.8.2) - 2025-03-11

### Fixed

- Fix cargo stylus replay [#226](https://github.com/OffchainLabs/stylus-sdk-rs/pull/226)

## [0.8.1](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.8.1) - 2025-02-21

### Fixed

- Add Reentrant Feature to Stylus Test When Enabled in SDK [#221](https://github.com/OffchainLabs/stylus-sdk-rs/pull/221)

## [0.8.0](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.8.0) - 2025-02-12

### Added

- Define a Host Trait for the Stylus SDK [#199](https://github.com/OffchainLabs/stylus-sdk-rs/pull/199)
- Define Initial WasmHost Implementation for Wasm Targets [#200](https://github.com/OffchainLabs/stylus-sdk-rs/pull/200)
- Use a Boxed, Dyn Host Trait for Contract Initialization [#203](https://github.com/OffchainLabs/stylus-sdk-rs/pull/203)
- Define Testing VM to Implement the Mock Trait [#204](https://github.com/OffchainLabs/stylus-sdk-rs/pull/204)
- Rename Stylus-Host to Stylus-Core and Make Calls Part of the VM [#206](https://github.com/OffchainLabs/stylus-sdk-rs/pull/206)
- Make Deployment Logic Part of the Host Trait [#207](https://github.com/OffchainLabs/stylus-sdk-rs/pull/207)
- Add unit tests to storage bytes [#213](https://github.com/OffchainLabs/stylus-sdk-rs/pull/213)
- Add Missing Methods to Host Trait [#210](https://github.com/OffchainLabs/stylus-sdk-rs/pull/210)
- Reduce Wasm Code Size Impact of Host Trait [#216](https://github.com/OffchainLabs/stylus-sdk-rs/pull/216)
- Add a Powerful Test VM [#212](https://github.com/OffchainLabs/stylus-sdk-rs/pull/212)

### Changed

- Deprecate Old Hostios and Improve TestVM Ergonomics [#209](https://github.com/OffchainLabs/stylus-sdk-rs/pull/209)
- Minimize calls to storage for bytes/string [#217](https://github.com/OffchainLabs/stylus-sdk-rs/pull/217)
- Make CI fail for clippy warnings [#220](https://github.com/OffchainLabs/stylus-sdk-rs/pull/220)
- v0.8.0 Release Candidate [#218](https://github.com/OffchainLabs/stylus-sdk-rs/pull/218)

### Fixed

- Fix storage bytes set-len when shrinking [#211](https://github.com/OffchainLabs/stylus-sdk-rs/pull/211)
- Fix examples and doctest [#219](https://github.com/OffchainLabs/stylus-sdk-rs/pull/219)

## [0.7.0](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.7.0) - 2025-02-03

### Added

- `impl From<alloy_primitives::Bytes> for stylus_sdk::Bytes`
- Support for integer types from `alloy_primitives`
- Fallback/receive functionality for routers created using `#[public]`

### Changed

- Upgrade alloy dependency to `0.8.14`
- Allow struct references within `sol_interface!` macro
- `pub` structs in `sol_interface!` macro
- Refactor of proc macros for better maintainability and testability


## [0.6.0](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.6.0) - 2024-08-30

### Breaking Changes

- `#[selector(id = ...)]` syntax has been removed to avoid misleading contracts
  from being implemented.
- Several methods in `RawDeploy` which were not fully implemented yet
- `#[pure]`, `#[view]` and `#[write]` attributes have been removed in favor of
  using arguments to infer state mutability.
- `stylus-sdk` now ships with `mini-alloc` enabled by default. This means that
  a `#[global_allocator]` should not be declared in most cases. If a custom
  allocator is still needed the `mini-alloc` should be disabled (enabled by
  default).
- `StorageU1` and `StorageI1` types have been removed.

### Deprecated

- The `#[external]` macro is now deprecated in favor of `#[public]` which
  provides the same funcitonality.
- The `#[solidity_storage]` macro is now deprecated in favor of `#[storage]`
  which provides the same functionality.

### Changed

- Ensure consistency between proc macros when parsing attributes.
- Update `sol_interface!` macro to report errors when using Solidity features
  which have not yet been implemented.

### Fixed

- Properly encode bytes when calling external contracts.
- Properly encode BYtes and strings in return types.
- Bytes type now works properly in `export-abi`.
- `export-abi` now works for contracts with no functions with returned values.
- Off-by-one error when storing strings with length 32.
- Interfaces in `sol_interface!` no longer incorrectly inherit functions from
  previous definitions.

### Documentation

- Various documentation updates for clarity.
- Cleaned up typos and moved TODOs to the github issue tracker.

### Security

- Function signatures which generate the same selector values will now fail
  at compile-time to avoid misleading contract calls.

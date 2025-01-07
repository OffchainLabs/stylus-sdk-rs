# Changelog

These crates follow [semver](https://semver.org).


## [0.7.0](https://github.com/OffchainLabs/stylus-sdk-rs/releases/tag/v0.6.0) - 2024-01-06

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

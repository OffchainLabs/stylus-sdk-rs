# stylus_proc

Procedural Macros for Stylus SDK

## Macro usage

Macro usage should be done through the [stylus-sdk] crate. Refer to the
[documentation] for additional information and examples.

## Development Considerations

### Error handling

The [proc_macro_error] crate is used for error handling to ensure consistency
across rust versions and for convenience.

Prefer [emit_error!] where possible to allow multiple errors to be displayed to
the user. If an error is reached where compilation must be halted, [abort!]
should be used instead.

### Testing

Procedural macro implementations should be written in a way that returns AST
data structures from the [syn] crate before converting them to
[proc_macro::TokenStream]. This allows the implementation to be unit tested
within its module to ensure the generated code is as expected.

The [trybuild] crate is used to write test cases which should fail to compile.
These tests are located in [tests/fail/] directory.

[stylus-sdk]: https://crates.io/crates/stylus-sdk
[documentation]: https://crates.io/crates/stylus-proc
[proc_macro_error]: https://crates.io/crates/proc-macro-error
[emit_error!]: https://docs.rs/proc-macro-error/latest/proc_macro_error/macro.emit_error.html
[abort!]: https://docs.rs/proc-macro-error/latest/proc_macro_error/macro.abort.html
[syn]: https://crates.io/crates/syn
[proc_macro::TokenStream]: https://docs.rs/proc-macro/latest/proc_macro/struct.TokenStream.html
[trybuild]: https://crates.io/crates/trybuild

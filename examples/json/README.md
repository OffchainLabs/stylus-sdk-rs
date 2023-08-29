# Json parser contract

This provides a contract interface for json parsing.
input format:
    PATH:JSON

PATH into the json is delimited by "|", each key is either a number or string inside double quotes, and the last key is either the word String or U64 (no quotes).
Value returned is either string or u64 according to the request.

## running internal tests
```
cd lib
cargo test
```

## compiling

```
cd contract
cargo stylus check
cargo stylus deploy
```

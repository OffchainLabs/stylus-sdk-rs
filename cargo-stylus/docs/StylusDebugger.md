# Stylus Debugger

This file explains stylus debugging capabilities via `stylusdb`.

## How to build it?

To use this, you need to install `stylusdb` tool from [stylusdb](https://github.com/walnuthq/stylusdb).

Install `cargo-stylus` from crates.io:

```bash
cargo install cargo-stylus
```

Note: This will install the binary as `cargo-stylus`. Use `cargo stylus` in all commands below.

## How to run it?

Lets use https://github.com/OffchainLabs/stylus-hello-world as an example.

In one terminal, start debug node:

```bash
docker run -it --rm --name nitro-dev -p 8547:8547 offchainlabs/nitro-node:v3.5.3-rc.3-653b078 --dev --http.addr 0.0.0.0 --http.api=net,web3,eth,arb,arbdebug,debug
```

In another terminal, compile and deploy the example:

```bash
git clone https://github.com/OffchainLabs/stylus-hello-world
cd stylus-hello-world
export RPC_URL=http://localhost:8547
export PRIV_KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
cargo stylus deploy --private-key=$PRIV_KEY --endpoint=$RPC_URL
```

and you can expect the output like this
```text
...
deployed code at address: 0xda52b25ddb0e3b9cc393b0690ac62245ac772527
deployment tx hash: 0x307b1d712840327349d561dea948d957362d5d807a1dfa87413023159cbb23f2
wasm already activated!

NOTE: We recommend running cargo stylus cache bid da52b25ddb0e3b9cc393b0690ac62245ac772527 0 to cache your activated contract in ArbOS.
Cached contracts benefit from cheaper calls. To read more about the Stylus contract cache, see
https://docs.arbitrum.io/stylus/concepts/stylus-cache-manager
$ export ADDR=0xda52b25ddb0e3b9cc393b0690ac62245ac772527
$ cast send --rpc-url=$RPC_URL --private-key=$PRIV_KEY $ADDR "increment()"
blockHash            0x3f6bea10728836b1f2c37e2aff3b69b1a7175b7607c8dc9df93aa3b4911536ed
blockNumber          5
contractAddress      
cumulativeGasUsed    992585
effectiveGasPrice    100000000
from                 0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
gasUsed              992585
logs                 []
logsBloom            0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
root                 
status               1 (success)
transactionHash      0x88b0ad9daa0b701d868a5f9a0132db7c0402178ba44ed8dec4ba76784c7194fd
transactionIndex     1
type                 2
blobGasPrice         
blobGasUsed          
authorizationList    
to                   0xA6E41fFD769491a42A6e5Ce453259b93983a22EF
gasUsedForL1         936000
l1BlockNumber        0
timeboosted          false
```

### Run `replay` command

This is the way of using existing `replay` option, that will attach to either `lldb` or `gdb`:

```bash
cargo stylus replay \
  --tx=0x88b0ad9daa0b701d868a5f9a0132db7c0402178ba44ed8dec4ba76784c7194fd \
  --endpoint=$RPC_URL
1 location added to breakpoint 1
warning: This version of LLDB has no plugin for the language "rust". Inspection of frame variables will be limited.
Process 9256 stopped
* thread #1, name = 'main', queue = 'com.apple.main-thread', stop reason = breakpoint 1.1
    frame #0: 0x000000010102cbb8 libstylus_hello_world.dylib`user_entrypoint(len=4) at lib.rs:33:5
   30  	// Define some persistent storage using the Solidity ABI.
   31  	// `Counter` will be the entrypoint.
   32  	sol_storage! {
-> 33  	    #[entrypoint]
   34  	    pub struct Counter {
   35  	        uint256 number;
   36  	    }
Target 0: (cargo-stylus) stopped.
Process 9256 launched: '~/.cargo/bin/cargo-stylus' (arm64)
(lldb) c
Process 9256 resuming
call completed successfully
Process 9256 exited with status = 0 (0x00000000) 
(lldb) q
```

### Run `usertrace` command

We have introduced a new `cargo` option called `usertrace`, that uses similar technology as `replay` option, but it rather attaches to `stylusdb`, instead of well known debuggers.

First, make sure you installed `colorama` package:

```bash
$ python3 -m venv myvenv
$ source ./myvenv/bin/activate
(myvenv) $ pip3 install colorama
```

We have introduced a new `cargo` option called `usertrace`, that uses similar technology as `replay` option, but it rather attaches to `stylusdb`, instead of well known debuggers.

``` bash
$ cargo stylus usertrace \
  --tx=0x88b0ad9daa0b701d868a5f9a0132db7c0402178ba44ed8dec4ba76784c7194fd \
  --endpoint=$RPC_URL
=== STYLUS FUNCTION CALL TREE ===
└─ #1 stylus_hello_world::__stylus_struct_entrypoint::h09ecd85e5c55b994 (lib.rs:33)
    input = size=4
    <anon> = stylus_sdk::host::VM { 0=<unavailable> }
  └─ #2 stylus_hello_world::Counter::increment::h5b9fb276c23de4f4 (lib.rs:64)
      self = 0x000000016fdeaa78
    └─ #3 stylus_hello_world::Counter::set_number::h5bd2c4836637ecb9 (lib.rs:49)
        self = 0x000000016fdeaa78
        new_number = ruint::Uint<256, 4> { limbs=unsigned long[4] { [0]=1, [1]=0, [2]=0, [3]=0 } }
```

In your terminal, it will look as:

<img width="699" alt="Screenshot 2025-04-14 at 13 09 47" src="https://github.com/user-attachments/assets/45ea3aaa-afa7-48fe-a832-7bf878903a6b" />

You may see the calltrace in form of JSON in:

```
/tmp/lldb_function_trace.json
```

By default, it does not follow functions from `stylus_sdk::`, if you want to see those, use `--verbose-usertrace` option, e.g.:

```bash
$ cargo stylus usertrace \
  --tx=0x88b0ad9daa0b701d868a5f9a0132db7c0402178ba44ed8dec4ba76784c7194fd \
  --endpoint=$RPC_URL --verbose-usertrace
```

Or, if you want to track calls from other libraries, just use `--trace-external-usertrace` as follows:

```bash
cargo stylus usertrace \
  --tx=0x88b0ad9daa0b701d868a5f9a0132db7c0402178ba44ed8dec4ba76784c7194fd \
  --endpoint=$RPC_URL --verbose-usertrace --trace-external-usertrace="std,core,other_contract"
```

and it will track calls from `std::`, `core` and `other_contract::`.

### Run `replay` option with `stylusdb`

To use `stylusdb`, specify `--debugger stylusdb`.

```bash
$ cargo stylus replay --debugger stylusdb --tx <TX_HASH> [other args]
```

If you want to debug multi-contract transaction, use:

```bash
$ cargo stylus replay --debugger stylusdb --tx <TX_HASH> \
  --contracts ADDR:PATH,0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e:../ \
  --endpoint=$RPC_URL
...
(stylusdb) stylus-contract breakpoint 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e external_contract::ServiceContract::increment
Set breakpoint on external_contract::ServiceContract::increment in contract 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e (ID: 3, 1 locations)
...
```

### Setting Breakpoints in StylusDB

StylusDB provides the `stylus-contract` command for managing breakpoints in multi-contract debugging sessions. Here are the available commands and examples:

#### Adding Contracts

Before setting breakpoints, you need to add the contract to the debugger:

```bash
(stylusdb) stylus-contract add 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF ./target/debug/libmy_contract.dylib
Added contract 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF with library ./target/debug/libmy_contract.dylib
```

#### Setting Breakpoints

Set breakpoints on specific functions within a contract:

```bash
# Break on a specific function
(stylusdb) stylus-contract breakpoint 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF stylus_hello_world::Counter::increment
Set breakpoint on stylus_hello_world::Counter::increment in contract 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF (ID: 1, 1 locations)

# Break on the entrypoint
(stylusdb) stylus-contract breakpoint 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF user_entrypoint
Set breakpoint on user_entrypoint in contract 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF (ID: 2, 1 locations)

# Break on internal functions
(stylusdb) stylus-contract breakpoint 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF stylus_hello_world::Counter::set_number
Set breakpoint on stylus_hello_world::Counter::set_number in contract 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF (ID: 3, 1 locations)
```

#### Other Useful Commands

```bash
# List all registered contracts
(stylusdb) stylus-contract list
Registered contracts:
  0xA6E41fFD769491a42A6e5Ce453259b93983a22EF -> ./target/debug/libmy_contract.dylib (3 breakpoints)
  0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e -> ../external/target/debug/libexternal.dylib (1 breakpoint)

# Show the current call stack
(stylusdb) stylus-contract stack
Call stack: main -> 0xA6E41fFD769491a42A6e5Ce453259b93983a22EF -> 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e

# Switch debugging context to a specific contract
(stylusdb) stylus-contract context 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e
Switched context to contract 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e
Module: ../external/target/debug/libexternal.dylib

# Show current context
(stylusdb) stylus-contract context show
Current context: 0xe1080224B632A93951A7CFA33EeEa9Fd81558b5e
```

#### Standard LLDB Commands

In addition to `stylus-contract` commands, you can use standard LLDB commands:

```bash
# Continue execution
(stylusdb) c

# Step over
(stylusdb) n

# Step into
(stylusdb) s

# Print variable
(stylusdb) p number

# Print with formatting (for Rust types)
(stylusdb) expr -f hex -- number.limbs[0]

# Show backtrace
(stylusdb) bt

# List breakpoints
(stylusdb) breakpoint list

# Delete a breakpoint
(stylusdb) breakpoint delete 1

# Quit debugger
(stylusdb) q
```

### Debugging Transactions with Solidity Contract Calls

When debugging transactions that involve calls from Stylus to Solidity contracts, you can use the `--addr-solidity` flag to mark specific addresses as Solidity contracts:

```bash
$ cargo stylus replay --debugger stylusdb --tx <TX_HASH> \
  --addr-solidity=0xda52b25ddb0e3b9cc393b0690ac62245ac772527 \
  --endpoint=$RPC_URL
```

When the debugger encounters a call to a Solidity contract, it will display:

```
════════ Solidity Contract Call ════════
Contract: 0xda52b25ddb0e3b9cc393b0690ac62245ac772527
Function selector: 0xd09de08a (increment())
NOTE: This is a Solidity contract - skipping to next contract
```

The debugger will:
- Show the Solidity contract address
- Display the function selector (first 4 bytes of calldata)
- Attempt to decode the function name using 4byte.directory (if available)
- Continue execution after the Solidity call returns

This allows you to trace execution flow across mixed Stylus/Solidity transactions, even though source-level debugging is only available for Stylus contracts.

#### Important Note on Contract Paths

When specifying contracts with the `--contracts` flag, you can only provide directory paths for Rust/Stylus contracts. Solidity contracts do not support full debugging and cannot be built from source directories.

**Incorrect usage (will fail):**
```bash
# This will error because ../erc20 is a Solidity contract directory
cargo stylus replay --debugger stylusdb \
  --tx=0xb590941f5de2a2164b76143ef4ca9d27df2d7c718c058fd2bbef4ac56b72d149 \
  --contracts 0xa6e41ffd769491a42a6e5ce453259b93983a22ef:.,0x1294b86822ff4976BfE136cB06CF43eC7FCF2574:../erc20
```

This will produce an error:
```
error: could not find `Cargo.toml` in `/path/to/erc20` or any parent directory
Error: failed to replay tx
Caused by:
    failed to open ../erc20/target/aarch64-apple-darwin/debug/: No such file or directory
```

**Correct usage:**
```bash
# Only specify the Solidity contract address without a path
cargo stylus replay --debugger stylusdb \
  --tx=0xb590941f5de2a2164b76143ef4ca9d27df2d7c718c058fd2bbef4ac56b72d149 \
  --contracts 0xa6e41ffd769491a42a6e5ce453259b93983a22ef:.,0x1294b86822ff4976BfE136cB06CF43eC7FCF2574
```

For Solidity contracts:
- Only provide the contract address (no `:path` suffix)
- The debugger will show the contract address and function selectors
- Source-level debugging is not available
- Execution will continue after Solidity calls return

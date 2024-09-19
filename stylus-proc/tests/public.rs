extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    #[payable]
    fn method() {}
}

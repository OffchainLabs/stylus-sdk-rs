use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref UINT_REGEX: Regex = Regex::new(r"^uint(\d+)$").unwrap();
    static ref INT_REGEX: Regex = Regex::new(r"^int(\d+)$").unwrap();
    static ref BYTES_REGEX: Regex = Regex::new(r"^bytes(\d+)$").unwrap();
}

pub fn is_sol_keyword(name: &str) -> bool {
    if let Some(caps) = UINT_REGEX.captures(name) {
        let bits: usize = caps[1].parse().unwrap();
        if bits.is_multiple_of(8) {
            return true;
        }
    }

    if let Some(caps) = INT_REGEX.captures(name) {
        let bits: usize = caps[1].parse().unwrap();
        if bits.is_multiple_of(8) {
            return true;
        }
    }

    if let Some(caps) = BYTES_REGEX.captures(name) {
        let bits: usize = caps[1].parse().unwrap();
        if bits <= 32 {
            return true;
        }
    }

    match name {
        // other types
        "address" | "bytes" | "bool" | "int" | "uint" => true,

        // other words
        "is" | "contract" | "interface" => true,

        // reserved keywords
        "after" | "alias" | "apply" | "auto" | "byte" | "case" | "copyof" | "default"
        | "define" | "final" | "implements" | "in" | "inline" | "let" | "macro" | "match"
        | "mutable" | "null" | "of" | "partial" | "promise" | "reference" | "relocatable"
        | "sealed" | "sizeof" | "static" | "supports" | "switch" | "typedef" | "typeof" | "var" => {
            true
        }
        _ => false,
    }
}

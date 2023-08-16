pub mod handler;

pub fn extract_call_parts(input: Vec<u8>) -> (u32, Vec<u8>) {
    let fn_selector: [u8; 4] = input[..4].try_into().unwrap();
    let fn_selector = u32::from_be_bytes(fn_selector);
    let args = input[4..].to_vec();
    (fn_selector, args)
}

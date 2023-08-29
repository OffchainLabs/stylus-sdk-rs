
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

use serde_json::Value;

pub fn json_main(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
    let input = String::from_utf8_lossy(&input);
    let (path, json) = input.split_once(":").unwrap();
    let mut value: Value = serde_json::from_str(json).unwrap();
    for elem in path.split("|") {
        if let Ok(index) = elem.parse::<usize>() {
                value = value.get_mut(index).unwrap().take();
                continue;
        }
        if elem.len() > 0 && elem.chars().nth(0).unwrap() == '\"' && elem.chars().last().unwrap() == '\"' {
            let fieldname = &elem[1..elem.len()-1];
            value = value[fieldname].take();
            continue;
        }
        if elem == "String" {
            return Ok(value.as_str().unwrap().as_bytes().to_vec());
        }
        if elem == "U64" {
            return Ok(value.as_u64().unwrap().to_be_bytes().to_vec());
        }
        panic!("invalid path");
    }
    panic!("invalid path");
}

#[cfg(test)]
mod tests {
    use crate::json_main;
    use alloc::borrow::ToOwned;

    fn expect_string(json: &str, path: &str, res: &str) {
        let mut input = path.to_owned();
        input.push_str("|String:");
        input.push_str(json);
        let ret = json_main(input.as_bytes().to_vec());
        assert_eq!(ret, Ok(res.as_bytes().to_vec()));
    }

    fn expect_u64(json: &str, path: &str, res: u64) {
        let mut input = path.to_owned();
        input.push_str("|U64:");
        input.push_str(json);
        let ret = json_main(input.as_bytes().to_vec());
        assert_eq!(ret, Ok(res.to_be_bytes().to_vec()));
    }

    #[test]
    fn simple() {
        let json = r#"
            {
                "a": "hello",
                "b": 12,
                "c": {
                    "d" : 7,
                    "e" : [1, 2, 3, 4]
                }
            }
        "#;
        expect_string(&json, "\"a\"", "hello");
        expect_u64(&json, "\"b\"", 12);
        expect_u64(&json, "\"c\"|\"d\"", 7);
        expect_u64(&json, "\"c\"|\"e\"|0", 1);
        expect_u64(&json, "\"c\"|\"e\"|1", 2);
        expect_u64(&json, "\"c\"|\"e\"|2", 3);
        expect_u64(&json, "\"c\"|\"e\"|3", 4);
    }
}
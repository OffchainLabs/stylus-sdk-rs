
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

pub fn parse_json(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
    let input = String::from_utf8_lossy(&input);
    let (path, json) = input.split_once(":").unwrap();
    let orig = json::parse(json).unwrap();
    let mut value = &orig;
    for path_elem in path.split("|") {
        if let Ok(index) = path_elem.parse::<usize>() {
            let mut i :usize = 0;
            for elem in value.members() {
                if i == index {
                    value = elem;
                    break;
                }
                i+=1;
            }
            if i != index {
                panic!("invalid index");
            }
            continue;
        }
        if path_elem.len() > 0 && path_elem.chars().nth(0).unwrap() == '\"' && path_elem.chars().last().unwrap() == '\"' {
            let fieldname = &path_elem[1..path_elem.len()-1];
            let mut found = false;
            for (entry_key, entry_value) in value.entries() {
                if entry_key == fieldname {
                    found = true;
                    value = entry_value;
                    break
                }
            }
            if !found {
                panic!("invalid key");
            }
            continue;
        }
        if path_elem == "String" {
            return Ok(value.as_str().unwrap().as_bytes().to_vec());
        }
        if path_elem == "U64" {
            return Ok(value.as_u64().unwrap().to_be_bytes().to_vec());
        }
        panic!("invalid path");
    }
    panic!("invalid path");
}

#[cfg(test)]
mod tests {
    use crate::parse_json;
    use alloc::borrow::ToOwned;

    fn expect_string(json: &str, path: &str, res: &str) {
        let mut input = path.to_owned();
        input.push_str("|String:");
        input.push_str(json);
        let ret = parse_json(input.as_bytes().to_vec());
        assert_eq!(ret, Ok(res.as_bytes().to_vec()));
    }

    fn expect_u64(json: &str, path: &str, res: u64) {
        let mut input = path.to_owned();
        input.push_str("|U64:");
        input.push_str(json);
        let ret = parse_json(input.as_bytes().to_vec());
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
pub mod u256;
pub mod address;
pub mod hash;
pub mod gas;

pub fn read_hex(s: &str) -> Option<Vec<u8>> {
    if s.starts_with("0x") {
        return read_hex(&s[2..s.len()]);
    }

    let mut res = Vec::<u8>::new();

    let mut cur = 0;
    let mut len = 0;
    for c in s.chars() {
        len += 1;
        let v_option = c.to_digit(16);
        if v_option.is_none() {
            return None;
        }
        let v = v_option.unwrap();
        if len == 1 {
            cur += v * 16;
        } else { // len == 2
            cur += v;
        }
        if len == 2 {
            res.push(cur as u8);
            cur = 0;
            len = 0;
        }
    }

    return Some(res);
}

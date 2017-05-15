pub mod bigint;
pub mod address;
pub mod gas;
pub mod opcode;

#[derive(Debug)]
pub enum ParseHexError {
    InvalidCharacter,
    TooLong,
    TooShort,
    Other
}

pub fn read_hex(s: &str) -> Result<Vec<u8>, ParseHexError> {
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
            return Err(ParseHexError::InvalidCharacter);
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

    return Ok(res);
}

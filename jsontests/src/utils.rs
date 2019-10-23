use std::str::FromStr;
use primitive_types::{U256, H256, H160};

pub fn unwrap_to_u256(s: &str) -> U256 {
	if s.starts_with("0x") {
		U256::from_str(&s[2..]).unwrap()
	} else {
		U256::from_dec_str(s).unwrap()
	}
}

pub fn unwrap_to_h256(s: &str) -> H256 {
	assert!(s.starts_with("0x"));
	H256::from_str(&s[2..]).unwrap()
}

pub fn unwrap_to_h160(s: &str) -> H160 {
	assert!(s.starts_with("0x"));
	H160::from_str(&s[2..]).unwrap()
}

pub fn unwrap_to_vec(s: &str) -> Vec<u8> {
	assert!(s.starts_with("0x"));
	hex::decode(&s[2..]).unwrap()
}

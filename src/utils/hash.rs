use utils::u256::U256;
use utils::read_hex;

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct H256([u8; 32]);

impl Default for H256 {
    fn default() -> H256 {
        H256([0u8; 32])
    }
}

impl Into<U256> for H256 {
    fn into(self) -> U256 {
        U256::from(self.0.as_ref())
    }
}

impl H256 {
    pub fn from_str(s: &str) -> Option<H256> {
        let v = read_hex(s);
        if v.is_none() { return None; }
        let v = v.unwrap();

        if v.len() > 32 {
            None
        } else {
            let mut a = [0u8; 32];

            for i in 0..v.len() {
                let j = i + (32 - v.len());
                a[j] = v[i];
            }

            Some(H256(a))
        }
    }
}

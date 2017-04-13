use utils::u256::U256;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Address([u8; 20]);

impl Default for Address {
    fn default() -> Address {
        Address([0u8; 20])
    }
}

impl Into<U256> for Address {
    fn into(self) -> U256 {
        U256::from(self.0.as_ref())
    }
}

impl From<U256> for Option<Address> {
    fn from(mut val: U256) -> Option<Address> {
        let max: U256 = U256::one() << 161;
        if val >= max {
            None
        } else {
            let mut i = 20;
            let mut a = [0u8; 20];

            while i != 0 {
                let u: u64 = (val & 0xFF.into()).into();
                a[i-1] = u as u8;

                i -= 1;
                val = val >> 8;
            }

            Some(Address(a))
        }
    }
}

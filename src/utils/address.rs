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

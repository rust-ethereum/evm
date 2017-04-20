mod algorithms;
mod m256;
mod mi256;
mod u256;

pub use self::m256::M256;
pub use self::u256::{U256, ParseU256Error};
pub use self::mi256::MI256;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Sign {
    Minus,
    NoSign,
    Plus,
}

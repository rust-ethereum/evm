mod algorithms;
mod m256;
mod mi256;
mod u256;
mod u512;

pub use self::m256::M256;
pub use self::u256::U256;
pub use self::mi256::MI256;
pub use self::u512::U512;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
/// Sign of an integer.
pub enum Sign {
    Minus,
    NoSign,
    Plus,
}

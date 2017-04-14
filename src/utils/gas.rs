use std::ops::{Add, AddAssign, Sub, SubAssign};
use utils::u256::U256;

#[derive(Clone, Copy, Debug)]
pub struct Gas(isize);

impl Gas {
    pub fn zero() -> Gas { Gas(0) }
    pub fn is_valid(&self) -> bool { self.0 >= 0 }
}

impl From<isize> for Gas {
    fn from(val: isize) -> Gas { Gas(val) }
}

impl From<U256> for Gas {
    fn from(val: U256) -> Self {
        let u: usize = val.into();
        Gas::from(u as isize)
    }
}

impl Into<U256> for Gas {
    fn into(self) -> U256 {
        assert!(self.is_valid());
        (self.0 as u64).into()
    }
}

impl Add for Gas {
    type Output = Gas;

    fn add(self, other: Gas) -> Gas {
        Gas(self.0 + other.0)
    }
}

impl AddAssign for Gas {
    fn add_assign(&mut self, other: Gas) {
        self.0 += other.0
    }
}

impl Sub for Gas {
    type Output = Gas;

    fn sub(self, other: Gas) -> Gas {
        Gas(self.0 - other.0)
    }
}

impl SubAssign for Gas {
    fn sub_assign(&mut self, other: Gas) {
        self.0 -= other.0
    }
}

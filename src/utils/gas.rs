use std::ops::{Add, AddAssign, Sub, SubAssign};
use utils::u256::U256;
use utils::read_hex;

#[derive(Clone, Copy, Debug)]
pub struct Gas(isize);

impl Gas {
    pub fn zero() -> Gas { Gas(0) }
    pub fn is_valid(&self) -> bool { self.0 >= 0 }

    pub fn from_str(s: &str) -> Option<Gas> {
        let v = read_hex(s);
        if v.is_none() { return None; }
        let v = v.unwrap();

        let mut g: isize = 0;
        for i in 0..v.len() {
            let j = v.len() - i - 1;
            g += (v[i] as isize) << (j * 8);
        }
        Some(Gas(g))
    }
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

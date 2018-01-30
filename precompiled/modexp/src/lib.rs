extern crate bigint;
extern crate num_bigint;
extern crate evm;

#[cfg(test)]
extern crate hexutil;

use std::rc::Rc;
use bigint::{Gas, U256};

use evm::Precompiled;
use evm::errors::{OnChainError, RuntimeError, NotSupportedError};

pub static MODEXP_PRECOMPILED: ModexpPrecompiled = ModexpPrecompiled;

pub struct ModexpPrecompiled;
impl Precompiled for ModexpPrecompiled {
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        use std::cmp;
        use num_bigint::BigUint;

        fn adjusted_exponent_length(exponent_length: U256, base_length: U256, data: &[u8]) -> U256 {
            let mut exp32_arr = Vec::new();
            for i in 0..32 {
                if U256::from(96) + base_length + U256::from(i) >= U256::from(data.len()) {
                    exp32_arr.push(0u8);
                } else {
                    let base_length_usize: usize = base_length.as_usize();
                    let data_i: usize = 96 + base_length_usize + i;
                    exp32_arr.push(data[data_i]);
                }
            }
            let exp32 = U256::from(exp32_arr.as_slice());

            if exponent_length <= U256::from(32) && exp32 == U256::zero() {
                U256::zero()
            } else if exponent_length <= U256::from(32) {
                U256::from(exp32.bits())
            } else {
                U256::from(8) * (exponent_length - U256::from(32)) + U256::from(exp32.bits())
            }
        }

        fn mult_complexity(x: U256) -> Result<U256, RuntimeError> {
            if x <= U256::from(64) {
                Ok(x * x)
            } else if x <= U256::from(1024) {
                Ok(x * x / U256::from(4) + U256::from(96) * x - U256::from(3072))
            } else {
                let (sqr, o) = x.overflowing_mul(x);
                if o {
                    Err(RuntimeError::OnChain(OnChainError::EmptyGas))
                } else {
                    Ok(sqr / U256::from(16) + U256::from(480) * x - U256::from(199680))
                }
            }
        }

        // Padding data to be at least 32 * 3 bytes.
        let mut data: Vec<u8> = data.into();
        while data.len() < 32 * 3 {
            data.push(0);
        }

        let base_length = U256::from(&data[0..32]);
        let exponent_length = U256::from(&data[32..64]);
        let modulus_length = U256::from(&data[64..96]);

        let op1 = mult_complexity(cmp::max(modulus_length, base_length))?;
        let op2 = cmp::max(adjusted_exponent_length(exponent_length, base_length, &data), U256::from(1)) / U256::from(20);
        let (r, o) = op1.overflowing_mul(op2);
        if o {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }
        let gas: Gas = r.into();

        if gas > gas_limit {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }

        if base_length > U256::from(usize::max_value()) ||
            exponent_length > U256::from(usize::max_value()) ||
            modulus_length > U256::from(usize::max_value())
        {
            return Err(RuntimeError::NotSupported(NotSupportedError::MemoryIndexNotSupported));
        }

        let base_length: usize = base_length.as_usize();
        let exponent_length: usize = exponent_length.as_usize();
        let modulus_length: usize = modulus_length.as_usize();

        let mut base_arr = Vec::new();
        let mut exponent_arr = Vec::new();
        let mut modulus_arr = Vec::new();

        for i in 0..base_length {
            if 96 + i >= data.len() {
                base_arr.push(0u8);
            } else {
                base_arr.push(data[96 + i]);
            }
        }
        for i in 0..exponent_length {
            if 96 + base_length + i >= data.len() {
                exponent_arr.push(0u8);
            } else {
                exponent_arr.push(data[96 + base_length + i]);
            }
        }
        for i in 0..modulus_length {
            if 96 + base_length + exponent_length + i >= data.len() {
                modulus_arr.push(0u8);
            } else {
                modulus_arr.push(data[96 + base_length + exponent_length + i]);
            }
        }

        let base = BigUint::from_bytes_be(&base_arr);
        let exponent = BigUint::from_bytes_be(&exponent_arr);
        let modulus = BigUint::from_bytes_be(&modulus_arr);

        let mut result = base.modpow(&exponent, &modulus).to_bytes_be();
        assert!(result.len() <= modulus_length);
        while result.len() < modulus_length {
            result.insert(0, 0u8);
        }

        Ok((gas, Rc::new(result)))
    }
}

#[cfg(test)]
mod tests {
    use ::*;
    use bigint::*;
    use hexutil::*;

    #[test]
    fn spec_test1() {
        let input = read_hex("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002003fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2efffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f").unwrap();
        let (_, output) = MODEXP_PRECOMPILED.gas_and_step(&input, Gas::from(10000000usize)).unwrap();
        let expected = read_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(expected, Rc::try_unwrap(output).unwrap());
    }

    #[test]
    fn spec_test2() {
        let input = read_hex("000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2efffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f").unwrap();
        let (_, output) = MODEXP_PRECOMPILED.gas_and_step(&input, Gas::from(10000000usize)).unwrap();
        let expected = read_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(expected, Rc::try_unwrap(output).unwrap());
    }

    #[test]
    fn spec_test3() {
        let input = read_hex("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd").unwrap();
        match MODEXP_PRECOMPILED.gas_and_step(&input, Gas::from(10000000usize)) {
            Ok(_) => panic!(),
            Err(_) => (),
        }
    }

    #[test]
    fn spec_test4() {
        let input = read_hex("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000002003ffff800000000000000000000000000000000000000000000000000000000000000007").unwrap();
        let (_, output) = MODEXP_PRECOMPILED.gas_and_step(&input, Gas::from(10000000usize)).unwrap();
        let expected = read_hex("3b01b01ac41f2d6e917c6d6a221ce793802469026d9ab7578fa2e79e4da6aaab").unwrap();
        assert_eq!(expected, Rc::try_unwrap(output).unwrap());
    }

    #[test]
    fn sepc_test5() {
        let input = read_hex("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000002003ffff80").unwrap();
        let (_, output) = MODEXP_PRECOMPILED.gas_and_step(&input, Gas::from(10000000usize)).unwrap();
        let expected = read_hex("3b01b01ac41f2d6e917c6d6a221ce793802469026d9ab7578fa2e79e4da6aaab").unwrap();
        assert_eq!(expected, Rc::try_unwrap(output).unwrap());
    }
}

extern crate bigint;
extern crate num_bigint;
extern crate evm;

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
                if U256::from(data.len()) < U256::from(96) + base_length + U256::from(i) {
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

        fn mult_complexity(x: U256) -> U256 {
            if x <= U256::from(64) {
                x * x
            } else if x <= U256::from(1024) {
                x * x / U256::from(4) + U256::from(96) * x - U256::from(3072)
            } else {
                x * x / U256::from(16) + U256::from(480) * x - U256::from(199680)
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

        let gas: Gas = (mult_complexity(cmp::max(modulus_length, base_length)) * cmp::max(adjusted_exponent_length(exponent_length, base_length, &data), U256::from(1)) / U256::from(20)).into();

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
            if data.len() < 96 + i {
                base_arr.push(0u8);
            } else {
                base_arr.push(data[96 + i]);
            }
        }
        for i in 0..exponent_length {
            if data.len() < 96 + base_length + i {
                exponent_arr.push(0u8);
            } else {
                exponent_arr.push(data[96 + base_length + i]);
            }
        }
        for i in 0..modulus_length {
            if data.len() < 96 + base_length + exponent_length + i {
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

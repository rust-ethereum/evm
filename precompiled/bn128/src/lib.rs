extern crate bigint;
extern crate bn;
extern crate evm;

use std::rc::Rc;
use bigint::{Gas, U256};

use evm::Precompiled;
use evm::errors::{OnChainError, RuntimeError, NotSupportedError};

pub struct Bn128AddPrecompiled;
impl Precompiled for Bn128AddPrecompiled {
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        use bn::{G1, AffineG1, Fq, Group};

        let gas = Gas::from(500usize);
        if gas > gas_limit {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }

        // Padding data to be at least 32 * 4 bytes.
        let mut data: Vec<u8> = data.into();
        while data.len() < 32 * 4 {
            data.push(0);
        }

        let px = Fq::from_slice(&data[0..32])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
        let py = Fq::from_slice(&data[32..64])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
        let qx = Fq::from_slice(&data[64..96])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
        let qy = Fq::from_slice(&data[96..128])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;

        let p = if px == Fq::zero() && py == Fq::zero() {
            G1::zero()
        } else {
            AffineG1::new(px, py).map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?.into()
        };
        let q = if qx == Fq::zero() && qy == Fq::zero() {
            G1::zero()
        } else {
            AffineG1::new(qx, qy).map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?.into()
        };

        let mut output = vec![0u8; 64];
        if let Some(ret) = AffineG1::from_jacobian(p + q) {
            ret.x().to_big_endian(&mut output[0..32]).unwrap();
            ret.y().to_big_endian(&mut output[32..64]).unwrap();
        }

        Ok((gas, Rc::new(output)))
    }
}

pub struct Bn128MulPrecompiled;
impl Precompiled for Bn128MulPrecompiled {
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        use bn::{G1, AffineG1, Fq, Fr, Group};

        let gas = Gas::from(40000usize);
        if gas > gas_limit {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }

        // Padding data to be at least 32 * 4 bytes.
        let mut data: Vec<u8> = data.into();
        while data.len() < 32 * 3 {
            data.push(0);
        }

        let px = Fq::from_slice(&data[0..32])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
        let py = Fq::from_slice(&data[32..64])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
        let fr = Fr::from_slice(&data[64..96])
            .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;

        let p = if px == Fq::zero() && py == Fq::zero() {
            G1::zero()
        } else {
            AffineG1::new(px, py).map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?.into()
        };

        let mut output = vec![0u8; 64];
        if let Some(ret) = AffineG1::from_jacobian(p * fr) {
            ret.x().to_big_endian(&mut output[0..32]).unwrap();
            ret.y().to_big_endian(&mut output[32..64]).unwrap();
        };

        Ok((gas, Rc::new(output)))
    }
}

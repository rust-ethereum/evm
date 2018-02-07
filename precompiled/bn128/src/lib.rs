extern crate bigint;
extern crate bn;
extern crate evm;

use std::rc::Rc;
use bigint::{Gas, U256};

use evm::Precompiled;
use evm::errors::{OnChainError, RuntimeError};

pub static BN128_ADD_PRECOMPILED: Bn128AddPrecompiled = Bn128AddPrecompiled;

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

pub static BN128_MUL_PRECOMPILED: Bn128MulPrecompiled = Bn128MulPrecompiled;

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

pub static BN128_PAIRING_PRECOMPILED: Bn128PairingPrecompiled = Bn128PairingPrecompiled;

pub struct Bn128PairingPrecompiled;
impl Precompiled for Bn128PairingPrecompiled {
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        use bn::{G1, AffineG1, Fq, Group, pairing, Gt, G2, Fq2, AffineG2};

        fn read_one(s: &[u8]) -> Result<(G1, G2), RuntimeError> {
            let ax = Fq::from_slice(&s[0..32])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
            let ay = Fq::from_slice(&s[32..64])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
            let bay = Fq::from_slice(&s[64..96])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
            let bax = Fq::from_slice(&s[96..128])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
            let bby = Fq::from_slice(&s[128..160])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;
            let bbx = Fq::from_slice(&s[160..192])
                .map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?;

            let ba = Fq2::new(bax, bay);
            let bb = Fq2::new(bbx, bby);

            let b = if ba.is_zero() && bb.is_zero() {
                G2::zero()
            } else {
                AffineG2::new(ba, bb).map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?.into()
            };
            let a = if ax.is_zero() && ay.is_zero() {
                G1::zero()
            } else {
                AffineG1::new(ax, ay).map_err(|_| RuntimeError::OnChain(OnChainError::EmptyGas))?.into()
            };

            Ok((a, b))
        }

        if data.len() % 192 != 0 {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }

        let ele_len = data.len() / 192;
        let gas = Gas::from(80000usize) * Gas::from(ele_len) + Gas::from(100000usize);
        if gas > gas_limit {
            return Err(RuntimeError::OnChain(OnChainError::EmptyGas));
        }

        let mut acc = Gt::one();
        for i in 0..ele_len {
            let (a, b) = read_one(&data[i*192..i*192+192])?;
            acc = acc * pairing(a, b);
        }

        let result = if acc == Gt::one() {
            U256::from(1)
        } else {
            U256::zero()
        };

        let mut output = vec![0u8; 32];
        result.to_big_endian(&mut output);

        Ok((gas, Rc::new(output)))
    }
}

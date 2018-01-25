#[cfg(not(feature = "std"))]
use alloc::Vec;

#[cfg(not(feature = "std"))] use alloc::rc::Rc;
#[cfg(feature = "std")] use std::rc::Rc;

use bigint::Gas;
#[cfg(all(feature = "std", any(feature = "rust-secp256k1", feature = "c-secp256k1")))] use std::cmp::min;
#[cfg(all(not(feature = "std"), any(feature = "rust-secp256k1", feature = "c-secp256k1")))] use core::cmp::min;

use errors::{RuntimeError, OnChainError};
use sha2::Sha256;
#[cfg(any(feature = "rust-secp256k1", feature = "c-secp256k1"))]
use sha3::Keccak256;
use ripemd160::Ripemd160;
use digest::{Digest, FixedOutput};

#[cfg(feature = "c-secp256k1")]
use secp256k1::{SECP256K1, RecoverableSignature, Message, RecoveryId, Error};
#[cfg(feature = "rust-secp256k1")]
use secp256k1::{recover, Message, RecoveryId, Signature, Error};

/// Represent a precompiled contract.
pub trait Precompiled: Sync {
    /// Step a precompiled contract based on the gas required.
    fn step(&self, _: &[u8]) -> Rc<Vec<u8>> {
        unimplemented!()
    }
    /// Gas needed for a given computation.
    fn gas(&self, _: &[u8]) -> Gas {
        unimplemented!()
    }
    /// Combine step and gas together, given the gas limit.
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        let gas = self.gas(data);
        if gas > gas_limit {
            Err(RuntimeError::OnChain(OnChainError::EmptyGas))
        } else {
            Ok((gas, self.step(data)))
        }
    }
}

/// ID precompiled contract.
pub struct IDPrecompiled;
impl Precompiled for IDPrecompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(15u64) +
            Gas::from(3u64) * gas_div_ceil(Gas::from(data.len()), Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Rc<Vec<u8>> {
        Rc::new(data.into())
    }
}
/// Static value of ID precompiled contract.
pub static ID_PRECOMPILED: IDPrecompiled = IDPrecompiled;

/// RIP160 precompiled contract.
pub struct RIP160Precompiled;
impl Precompiled for RIP160Precompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(600u64) +
            Gas::from(120u64) * gas_div_ceil(Gas::from(data.len()), Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Rc<Vec<u8>> {
        let mut ripemd = Ripemd160::default();
        ripemd.input(data);
        let fixed = ripemd.fixed_result();
        let mut result: [u8; 32] = [0u8; 32];
        for i in 0..20 {
            result[i + 12] = fixed[i];
        }
        Rc::new(result.as_ref().into())
    }
}
/// Static value of RIP160 precompiled contract.
pub static RIP160_PRECOMPILED: RIP160Precompiled = RIP160Precompiled;

/// SHA256 precompiled contract.
pub struct SHA256Precompiled;
impl Precompiled for SHA256Precompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(60u64) +
            Gas::from(12u64) * gas_div_ceil(Gas::from(data.len()),
                                            Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Rc<Vec<u8>> {
        let mut sha2 = Sha256::default();
        sha2.input(data);
        let fixed = sha2.fixed_result();
        let mut result: [u8; 32] = [0u8; 32];
        for i in 0..32 {
            result[i] = fixed[i];
        }
        Rc::new(result.as_ref().into())
    }
}
/// Static value of SHA256 precompiled contract.
pub static SHA256_PRECOMPILED: SHA256Precompiled = SHA256Precompiled;

/// ECREC precompiled contract.
pub struct ECRECPrecompiled;
#[cfg(any(feature = "c-secp256k1", feature = "rust-secp256k1"))]
impl Precompiled for ECRECPrecompiled {
    fn gas(&self, _: &[u8]) -> Gas {
        Gas::from(3000u64)
    }

    fn step(&self, datao: &[u8]) -> Rc<Vec<u8>> {
        let mut data = [0u8; 128];
        for i in 0..min(datao.len(), 128) {
            data[i] = datao[i];
        }
        match kececrec(&data) {
            Ok(mut ret) => {
                for i in 0..12 {
                    ret[i] = 0u8;
                }
                Rc::new(ret.as_ref().into())
            },
            Err(_) => Rc::new(Vec::new()),
        }
    }
}
#[cfg(all(not(feature = "c-secp256k1"), not(feature = "rust-secp256k1")))]
impl Precompiled for ECRECPrecompiled {
    fn gas_and_step(&self, _: &[u8], _: Gas) -> Result<(Gas, Rc<Vec<u8>>), RuntimeError> {
        use errors::NotSupportedError;

        Err(RuntimeError::NotSupported(NotSupportedError::PrecompiledNotSupported))
    }
}
/// Static value of ECREC precompiled contract.
pub static ECREC_PRECOMPILED: ECRECPrecompiled = ECRECPrecompiled;

fn gas_div_ceil(a: Gas, b: Gas) -> Gas {
    if a % b == Gas::zero() {
        a / b
    } else {
        a / b + Gas::from(1u64)
    }
}

#[cfg(feature = "c-secp256k1")]
fn kececrec(data: &[u8]) -> Result<[u8; 32], Error> {
    let message = Message::from_slice(&data[0..32])?;
    let recid_raw = match data[63] {
        27 | 28 if data[32..63] == [0; 31] => data[63] - 27,
        _ => return Err(Error::InvalidRecoveryId),
    };
    let recid = RecoveryId::from_i32(recid_raw as i32)?;
    let sig = RecoverableSignature::from_compact(&SECP256K1, &data[64..128], recid)?;
    let recovered = SECP256K1.recover(&message, &sig)?;
    let key = recovered.serialize_vec(&SECP256K1, false);

    let ret_generic = Keccak256::digest(&key[1..65]);
    let mut ret = [0u8; 32];

    for i in 0..32 {
        ret[i] = ret_generic[i];
    }

    Ok(ret)
}

#[cfg(feature = "rust-secp256k1")]
fn kececrec(data: &[u8]) -> Result<[u8; 32], Error> {
    let mut message_raw = [0u8; 32];
    for i in 0..32 {
        message_raw[i] = data[i];
    }
    let message = Message::parse(&message_raw);
    let recid_raw = match data[63] {
        27 | 28 if data[32..63] == [0; 31] => data[63] - 27,
        _ => return Err(Error::InvalidRecoveryId),
    };
    let recid = RecoveryId::parse(recid_raw)?;
    let mut sig_raw = [0u8; 64];
    for i in 0..64 {
        sig_raw[i] = data[64 + i];
    }
    let sig = Signature::parse(&sig_raw);
    let recovered = recover(&message, &sig, &recid)?;
    let key = recovered.serialize();

    let ret_generic = Keccak256::digest(&key[1..65]);
    let mut ret = [0u8; 32];

    for i in 0..32 {
        ret[i] = ret_generic[i];
    }

    Ok(ret)
}

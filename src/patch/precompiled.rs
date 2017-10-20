#[cfg(not(feature = "std"))]
use alloc::Vec;

use bigint::Gas;
#[cfg(feature = "std")] use std::cmp::min;

use errors::{RuntimeError, OnChainError};
#[cfg(feature = "std")]
use sha2::Sha256;
#[cfg(feature = "std")]
use sha3::Keccak256;
#[cfg(feature = "std")]
use ripemd160::Ripemd160;
#[cfg(feature = "std")]
use secp256k1::{SECP256K1, RecoverableSignature, Message, RecoveryId, Error};
#[cfg(feature = "std")]
use digest::{Digest, FixedOutput};

/// Represent a precompiled contract.
pub trait Precompiled: Sync {
    /// Step a precompiled contract based on the gas required.
    fn step(&self, _: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
    /// Gas needed for a given computation.
    fn gas(&self, _: &[u8]) -> Gas {
        unimplemented!()
    }
    /// Combine step and gas together, given the gas limit.
    fn gas_and_step(&self, data: &[u8], gas_limit: Gas) -> Result<(Gas, Vec<u8>), RuntimeError> {
        let gas = self.gas(data);
        if gas > gas_limit {
            Err(RuntimeError::OnChain(OnChainError::EmptyGas))
        } else {
            Ok((gas, self.step(data)))
        }
    }
}

#[cfg(feature = "std")]
/// ID precompiled contract.
pub struct IDPrecompiled;
#[cfg(feature = "std")]
impl Precompiled for IDPrecompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(15u64) +
            Gas::from(3u64) * gas_div_ceil(Gas::from(data.len()), Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Vec<u8> {
        data.into()
    }
}
#[cfg(feature = "std")]
pub static ID_PRECOMPILED: IDPrecompiled = IDPrecompiled;

#[cfg(feature = "std")]
/// RIP160 precompiled contract.
pub struct RIP160Precompiled;
#[cfg(feature = "std")]
impl Precompiled for RIP160Precompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(600u64) +
            Gas::from(120u64) * gas_div_ceil(Gas::from(data.len()), Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Vec<u8> {
        let mut ripemd = Ripemd160::default();
        ripemd.input(data);
        let fixed = ripemd.fixed_result();
        let mut result: [u8; 32] = [0u8; 32];
        for i in 0..20 {
            result[i + 12] = fixed[i];
        }
        result.as_ref().into()
    }
}
#[cfg(feature = "std")]
pub static RIP160_PRECOMPILED: RIP160Precompiled = RIP160Precompiled;

#[cfg(feature = "std")]
/// SHA256 precompiled contract.
pub struct SHA256Precompiled;
#[cfg(feature = "std")]
impl Precompiled for SHA256Precompiled {
    fn gas(&self, data: &[u8]) -> Gas {
        Gas::from(60u64) +
            Gas::from(12u64) * gas_div_ceil(Gas::from(data.len()),
                                            Gas::from(32u64))
    }

    fn step(&self, data: &[u8]) -> Vec<u8> {
        let mut sha2 = Sha256::default();
        sha2.input(data);
        let fixed = sha2.fixed_result();
        let mut result: [u8; 32] = [0u8; 32];
        for i in 0..32 {
            result[i] = fixed[i];
        }
        result.as_ref().into()
    }
}
#[cfg(feature = "std")]
pub static SHA256_PRECOMPILED: SHA256Precompiled = SHA256Precompiled;

#[cfg(feature = "std")]
/// ECREC precompiled contract.
pub struct ECRECPrecompiled;
#[cfg(feature = "std")]
impl Precompiled for ECRECPrecompiled {
    fn gas(&self, _: &[u8]) -> Gas {
        Gas::from(3000u64)
    }

    fn step(&self, datao: &[u8]) -> Vec<u8> {
        let mut data = [0u8; 128];
        for i in 0..min(datao.len(), 128) {
            data[i] = datao[i];
        }
        match kececrec(&data) {
            Ok(mut ret) => {
                for i in 0..12 {
                    ret[i] = 0u8;
                }
                ret.as_ref().into()
            },
            Err(_) => Vec::new(),
        }
    }
}
#[cfg(feature = "std")]
pub static ECREC_PRECOMPILED: ECRECPrecompiled = ECRECPrecompiled;

fn gas_div_ceil(a: Gas, b: Gas) -> Gas {
    if a % b == Gas::zero() {
        a / b
    } else {
        a / b + Gas::from(1u64)
    }
}

#[cfg(feature = "std")]
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

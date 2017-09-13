//! Functionality for precompiled accounts.

use bigint::{Address, Gas};

use errors::MachineError;
use ::{Memory, Machine, MachineStatus};

use std::str::FromStr;
use std::cmp::min;

use sha2::Sha256;
use sha3::Keccak256;
use ripemd160::Ripemd160;
use secp256k1::{SECP256K1, RecoverableSignature, Message, RecoveryId, Error};
use digest::{Digest, FixedOutput};

fn gas_div_ceil(a: Gas, b: Gas) -> Gas {
    if a % b == Gas::zero() {
        a / b
    } else {
        a / b + Gas::from(1u64)
    }
}

impl<M: Memory + Default> Machine<M> {
    /// Step a precompiled runtime. This function returns true if the
    /// runtime is indeed a precompiled address. Otherwise return
    /// false with state unchanged.
    pub fn step_precompiled(&mut self) -> bool {
        let ecrec_address = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let sha256_address = Address::from_str("0x0000000000000000000000000000000000000002").unwrap();
        let rip160_address = Address::from_str("0x0000000000000000000000000000000000000003").unwrap();
        let id_address = Address::from_str("0x0000000000000000000000000000000000000004").unwrap();

        if self.state.context.address == id_address {
            self.step_precompiled_id();
            return true;
        } else if self.state.context.address == rip160_address {
            self.step_precompiled_rip160();
            return true;
        } else if self.state.context.address == sha256_address {
            self.step_precompiled_sha256();
            return true;
        } else if self.state.context.address == ecrec_address {
            self.step_precompiled_ecrec();
            return true;
        } else {
            return false;
        }
    }

    fn step_precompiled_id(&mut self) {
        let gas = Gas::from(15u64) +
            Gas::from(3u64) * gas_div_ceil(Gas::from(self.state.context.data.len()),
                                           Gas::from(32u64));
        if gas > self.state.context.gas_limit {
            self.state.used_gas = self.state.context.gas_limit;
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
        } else {
            self.state.used_gas = gas;
            self.state.out = self.state.context.data.clone();
            self.status = MachineStatus::ExitedOk;
        }
    }

    fn step_precompiled_rip160(&mut self) {
        let gas = Gas::from(600u64) +
            Gas::from(120u64) * gas_div_ceil(Gas::from(self.state.context.data.len()),
                                             Gas::from(32u64));
        if gas > self.state.context.gas_limit {
            self.state.used_gas = self.state.context.gas_limit;
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
        } else {
            self.state.used_gas = gas;
            let mut ripemd = Ripemd160::default();
            ripemd.input(&self.state.context.data);
            let fixed = ripemd.fixed_result();
            let mut result: [u8; 32] = [0u8; 32];
            for i in 0..20 {
                result[i + 12] = fixed[i];
            }
            self.state.out = result.as_ref().into();
            self.status = MachineStatus::ExitedOk;
        }
    }

    fn step_precompiled_sha256(&mut self) {
        let gas = Gas::from(60u64) +
            Gas::from(12u64) * gas_div_ceil(Gas::from(self.state.context.data.len()),
                                            Gas::from(32u64));
        if gas > self.state.context.gas_limit {
            self.state.used_gas = self.state.context.gas_limit;
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
        } else {
            self.state.used_gas = gas;
            let mut sha2 = Sha256::default();
            sha2.input(&self.state.context.data);
            let fixed = sha2.fixed_result();
            let mut result: [u8; 32] = [0u8; 32];
            for i in 0..32 {
                result[i] = fixed[i];
            }
            self.state.out = result.as_ref().into();
            self.status = MachineStatus::ExitedOk;
        }
    }

    fn step_precompiled_ecrec(&mut self) {
        let gas = Gas::from(3000u64);
        if gas > self.state.context.gas_limit {
            self.state.used_gas = self.state.context.gas_limit;
            self.status = MachineStatus::ExitedErr(MachineError::EmptyGas);
        } else {
            self.state.used_gas = gas;
            let mut data = [0u8; 128];
            for i in 0..min(self.state.context.data.len(), 128) {
                data[i] = self.state.context.data[i];
            }
            match kececrec(&data) {
                Ok(mut ret) => {
                    for i in 0..12 {
                        ret[i] = 0u8;
                    }
                    self.state.out = ret.as_ref().into();
                },
                Err(_) => (),
            }
            self.status = MachineStatus::ExitedOk;
        }
    }
}

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

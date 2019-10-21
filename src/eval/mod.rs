#[macro_use]
mod macros;
mod system;

use crate::{Interrupt, Runtime, ExitReason, ExternalOpcode};

pub enum Control {
	Continue,
	Interrupt(Vec<Interrupt>),
	Exit(ExitReason)
}

pub fn eval(state: &mut Runtime, opcode: ExternalOpcode) -> Control {
	match opcode {
		ExternalOpcode::Sha3 => system::sha3(state),
		ExternalOpcode::Address => system::address(state),
		ExternalOpcode::Balance => system::balance(state),
		ExternalOpcode::Origin => system::origin(state),
		ExternalOpcode::Caller => system::caller(state),
		ExternalOpcode::CallValue => system::callvalue(state),
		ExternalOpcode::GasPrice => system::gasprice(state),
		ExternalOpcode::ExtCodeSize => system::extcodesize(state),
		ExternalOpcode::ExtCodeHash => system::extcodehash(state),
		ExternalOpcode::ExtCodeCopy => system::extcodecopy(state),
		ExternalOpcode::ReturnDataSize => system::returndatasize(state),
		ExternalOpcode::ReturnDataCopy => system::returndatacopy(state),
		ExternalOpcode::BlockHash => system::blockhash(state),
		ExternalOpcode::Coinbase => system::coinbase(state),
		ExternalOpcode::Timestamp => system::timestamp(state),
		ExternalOpcode::Number => system::number(state),
		ExternalOpcode::Difficulty => system::difficulty(state),
		ExternalOpcode::GasLimit => system::gaslimit(state),
		ExternalOpcode::SLoad => system::sload(state),
		ExternalOpcode::SStore => system::sstore(state),
		ExternalOpcode::Gas => unimplemented!(),
		ExternalOpcode::Log(n) => system::log(state, n),
		ExternalOpcode::Suicide => system::suicide(state),
		_ => unimplemented!(),
	}
}

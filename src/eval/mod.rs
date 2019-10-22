#[macro_use]
mod macros;
mod system;

use crate::{Handler, Runtime, ExitReason, ExternalOpcode};

pub enum Control<H: Handler> {
	Continue,
	CallInterrupt(H::CallInterrupt),
	CreateInterrupt(H::CreateInterrupt),
	Exit(ExitReason)
}

pub fn eval<H: Handler>(state: &mut Runtime, opcode: ExternalOpcode, handler: &mut H) -> Control<H> {
	match opcode {
		ExternalOpcode::Sha3 => system::sha3(state),
		ExternalOpcode::Address => system::address(state),
		ExternalOpcode::Balance => system::balance(state, handler),
		ExternalOpcode::Origin => system::origin(state),
		ExternalOpcode::Caller => system::caller(state),
		ExternalOpcode::CallValue => system::callvalue(state),
		ExternalOpcode::GasPrice => system::gasprice(state),
		ExternalOpcode::ExtCodeSize => system::extcodesize(state, handler),
		ExternalOpcode::ExtCodeHash => system::extcodehash(state, handler),
		ExternalOpcode::ExtCodeCopy => system::extcodecopy(state, handler),
		ExternalOpcode::ReturnDataSize => system::returndatasize(state),
		ExternalOpcode::ReturnDataCopy => system::returndatacopy(state),
		ExternalOpcode::BlockHash => system::blockhash(state, handler),
		ExternalOpcode::Coinbase => system::coinbase(state, handler),
		ExternalOpcode::Timestamp => system::timestamp(state, handler),
		ExternalOpcode::Number => system::number(state, handler),
		ExternalOpcode::Difficulty => system::difficulty(state, handler),
		ExternalOpcode::GasLimit => system::gaslimit(state, handler),
		ExternalOpcode::SLoad => system::sload(state, handler),
		ExternalOpcode::SStore => system::sstore(state, handler),
		ExternalOpcode::Gas => system::gas(state, handler),
		ExternalOpcode::Log(n) => system::log(state, n, handler),
		ExternalOpcode::Suicide => system::suicide(state, handler),
		_ => unimplemented!(),
	}
}

#[macro_use]
mod macros;
mod system;

use crate::{Handler, Runtime, ExitReason, ExternalOpcode, CallScheme};

pub enum Control<H: Handler> {
	Continue,
	CallInterrupt(H::CallInterrupt),
	CreateInterrupt(H::CreateInterrupt),
	Exit(ExitReason)
}

fn handle_other<H: Handler>(state: &mut Runtime, opcode: ExternalOpcode, handler: &mut H) -> Control<H> {
	match handler.other(
		opcode as u8,
		&mut state.machine
	) {
		Ok(()) => Control::Continue,
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn eval<H: Handler>(state: &mut Runtime, opcode: ExternalOpcode, handler: &mut H) -> Control<H> {
	match opcode {
		ExternalOpcode::Sha3 => system::sha3(state),
		ExternalOpcode::Address => system::address(state),
		ExternalOpcode::Balance => system::balance(state, handler),
		ExternalOpcode::SelfBalance => system::selfbalance(state, handler),
		ExternalOpcode::Origin => system::origin(state, handler),
		ExternalOpcode::Caller => system::caller(state),
		ExternalOpcode::CallValue => system::callvalue(state),
		ExternalOpcode::GasPrice => system::gasprice(state, handler),
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
		ExternalOpcode::Log0 => system::log(state, 0, handler),
		ExternalOpcode::Log1 => system::log(state, 1, handler),
		ExternalOpcode::Log2 => system::log(state, 2, handler),
		ExternalOpcode::Log3 => system::log(state, 3, handler),
		ExternalOpcode::Log4 => system::log(state, 4, handler),
		ExternalOpcode::Suicide => system::suicide(state, handler),
		ExternalOpcode::Create => system::create(state, false, handler),
		ExternalOpcode::Create2 => system::create(state, true, handler),
		ExternalOpcode::Call => system::call(state, CallScheme::Call, handler),
		ExternalOpcode::CallCode => system::call(state, CallScheme::CallCode, handler),
		ExternalOpcode::DelegateCall => system::call(state, CallScheme::DelegateCall, handler),
		ExternalOpcode::StaticCall => system::call(state, CallScheme::StaticCall, handler),
		ExternalOpcode::ChainId => system::chainid(state, handler),
		_ => handle_other(state, opcode, handler),
	}
}

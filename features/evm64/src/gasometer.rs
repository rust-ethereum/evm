use evm::{
	interpreter::{Control, ExitException, Machine, Opcode},
	standard::GasometerState,
};

macro_rules! try_or_fail {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(e) => return Control::Exit(e.into()),
		}
	};
}

pub fn eval<S, H, Tr>(machine: &mut Machine<S>, _handler: &mut H, position: usize) -> Control<Tr>
where
	S: AsRef<GasometerState> + AsMut<GasometerState>,
{
	const _G_BASE64: u64 = 1;
	const G_VERYLOW64: u64 = 2;
	const G_LOW64: u64 = 3;
	const G_MID64: u64 = 5;
	const G_HIGH64: u64 = 7;
	const G_EXP64_STATIC: u64 = 5;
	const G_EXP64_DYNAMIC: u64 = 25;

	const STATIC_COST_TABLE: [u64; 256] = {
		let mut table = [0; 256];
		table[Opcode::ADD.as_usize()] = G_VERYLOW64;
		table[Opcode::SUB.as_usize()] = G_VERYLOW64;
		table[Opcode::MUL.as_usize()] = G_LOW64;
		table[Opcode::DIV.as_usize()] = G_LOW64;
		table[Opcode::SDIV.as_usize()] = G_LOW64;
		table[Opcode::MOD.as_usize()] = G_LOW64;
		table[Opcode::SMOD.as_usize()] = G_LOW64;
		table[Opcode::ADDMOD.as_usize()] = G_MID64;
		table[Opcode::MULMOD.as_usize()] = G_MID64;
		table[Opcode::LT.as_usize()] = G_VERYLOW64;
		table[Opcode::GT.as_usize()] = G_VERYLOW64;
		table[Opcode::SLT.as_usize()] = G_VERYLOW64;
		table[Opcode::SGT.as_usize()] = G_VERYLOW64;
		table[Opcode::EQ.as_usize()] = G_VERYLOW64;
		table[Opcode::AND.as_usize()] = G_VERYLOW64;
		table[Opcode::OR.as_usize()] = G_VERYLOW64;
		table[Opcode::XOR.as_usize()] = G_VERYLOW64;
		table[Opcode::ISZERO.as_usize()] = G_VERYLOW64;
		table[Opcode::NOT.as_usize()] = G_VERYLOW64;
		table[Opcode::SHL.as_usize()] = G_VERYLOW64;
		table[Opcode::SHR.as_usize()] = G_VERYLOW64;
		table[Opcode::SAR.as_usize()] = G_VERYLOW64;
		table[Opcode::JUMP.as_usize()] = G_MID64;
		table[Opcode::JUMPI.as_usize()] = G_HIGH64;

		table
	};

	let opcode = Opcode(machine.code()[position]);

	try_or_fail!(machine
		.state
		.as_mut()
		.record_gas64(STATIC_COST_TABLE[opcode.as_usize()]));

	#[allow(clippy::single_match)]
	match opcode {
		Opcode::EXP => {
			let power = try_or_fail!(machine.stack.peek(1)).0[0];
			let cost = if power == 0 {
				G_EXP64_STATIC
			} else {
				try_or_fail!(G_EXP64_DYNAMIC
					.checked_mul(power.ilog2() as u64 / 8 + 1)
					.and_then(|dynamic| G_EXP64_STATIC.checked_add(dynamic))
					.ok_or(ExitException::OutOfGas))
			};
			try_or_fail!(machine.state.as_mut().record_gas64(cost));
		}
		_ => (),
	}

	Control::NoAction
}

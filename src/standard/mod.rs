mod config;
mod gasometer;

pub use self::config::Config;
pub use self::gasometer::Gasometer;

use primitive_types::U256;

pub struct RuntimeState<'config> {
	pub context: crate::Context,
	pub retbuf: Vec<u8>,
	pub gasometer: Gasometer<'config>,
}

impl<'config> crate::RuntimeState for RuntimeState<'config> {
	fn context(&self) -> &crate::Context {
		&self.context
	}

	fn retbuf(&self) -> &Vec<u8> {
		&self.retbuf
	}

	fn retbuf_mut(&mut self) -> &mut Vec<u8> {
		&mut self.retbuf
	}

	fn gas(&self) -> U256 {
		U256::from(self.gasometer.gas())
	}
}

pub type Machine<'config> = crate::Machine<RuntimeState<'config>>;

mod config;
mod gasometer;
mod invoker;

pub use self::config::Config;
pub use self::gasometer::{Gasometer, TransactGasometer};
pub use self::invoker::{Invoker, PrecompileSet, TransactArgs};

pub type Machine = crate::Machine<crate::RuntimeState>;
pub type Efn<H> = crate::Efn<crate::RuntimeState, H, crate::Opcode>;
pub type Etable<H, F = Efn<H>> = crate::Etable<crate::RuntimeState, H, crate::Opcode, F>;
pub type GasedMachine<G> = crate::GasedMachine<crate::RuntimeState, G>;

pub trait MergeableRuntimeState: AsRef<crate::RuntimeState> + AsMut<crate::RuntimeState> {
	fn substate(&self, runtime: crate::RuntimeState) -> Self;
	fn merge(&mut self, substate: Self, strategy: crate::MergeStrategy);
	fn new_transact_call(runtime: crate::RuntimeState) -> Self;
	fn new_transact_create(runtime: crate::RuntimeState) -> Self;
}

impl MergeableRuntimeState for crate::RuntimeState {
	fn substate(&self, runtime: crate::RuntimeState) -> Self {
		runtime
	}
	fn merge(&mut self, _substate: Self, _strategy: crate::MergeStrategy) {}
	fn new_transact_call(runtime: crate::RuntimeState) -> Self {
		runtime
	}
	fn new_transact_create(runtime: crate::RuntimeState) -> Self {
		runtime
	}
}

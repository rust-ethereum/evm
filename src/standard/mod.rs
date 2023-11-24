mod config;
mod gasometer;
mod invoker;

pub use self::config::Config;
pub use self::gasometer::{Gasometer, TransactGasometer};
pub use self::invoker::{EtableResolver, Invoker, PrecompileSet, Resolver, TransactArgs};

pub type Machine = crate::Machine<crate::RuntimeState>;
pub type Efn<H> = crate::Efn<crate::RuntimeState, H, crate::Opcode>;
pub type Etable<H, F = Efn<H>> = crate::Etable<crate::RuntimeState, H, crate::Opcode, F>;
pub type ColoredMachine<'etable, G, H, F = Efn<H>> =
	crate::ColoredMachine<crate::RuntimeState, G, &'etable Etable<H, F>>;

pub trait MergeableRuntimeState<M>:
	AsRef<crate::RuntimeState> + AsMut<crate::RuntimeState>
{
	fn substate(&self, runtime: crate::RuntimeState, parent: &M) -> Self;
	fn merge(&mut self, substate: Self, strategy: crate::MergeStrategy);
	fn new_transact_call(runtime: crate::RuntimeState) -> Self;
	fn new_transact_create(runtime: crate::RuntimeState) -> Self;
}

impl<M> MergeableRuntimeState<M> for crate::RuntimeState {
	fn substate(&self, runtime: crate::RuntimeState, _parent: &M) -> Self {
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

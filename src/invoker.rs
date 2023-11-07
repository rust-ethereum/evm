use crate::{Capture, ExitError, ExitResult, GasedMachine};

pub trait Invoker<S, G, H, Tr> {
	type Interrupt;
	type CallCreateTrapData;

	fn exit_trap_stack(
		&self,
		result: ExitResult,
		child: GasedMachine<S, G>,
		trap_data: Self::CallCreateTrapData,
		parent: &mut GasedMachine<S, G>,
		handler: &mut H,
	) -> Result<(), ExitError>;

	fn prepare_trap(
		&self,
		trap: Tr,
		machine: &mut GasedMachine<S, G>,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapData, ExitError>, Self::Interrupt>;

	fn enter_trap_stack(
		&self,
		trap_data: &Self::CallCreateTrapData,
		handler: &mut H,
	) -> Result<GasedMachine<S, G>, ExitError>;
}

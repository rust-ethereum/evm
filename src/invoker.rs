use crate::{Capture, ExitError, ExitResult, GasedMachine};

pub trait Invoker<S, G, H, Tr> {
	type Interrupt;
	type CallCreateTrapPrepareData;
	type CallCreateTrapEnterData;

	fn exit_trap_stack(
		&self,
		result: ExitResult,
		child: GasedMachine<S, G>,
		trap_data: Self::CallCreateTrapEnterData,
		parent: &mut GasedMachine<S, G>,
		handler: &mut H,
	) -> Result<(), ExitError>;

	/// The separation of `prepare_trap` and `enter_trap_stack` is to give an opportunity for the
	/// trait to return `Self::Interrupt`. When `Self::Interrupt` is `Infallible`, there's no
	/// difference whether a code is in `prepare_trap` or `enter_trap_stack`.
	fn prepare_trap(
		&self,
		trap: Tr,
		machine: &mut GasedMachine<S, G>,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapPrepareData, ExitError>, Self::Interrupt>;

	fn enter_trap_stack(
		&self,
		trap_data: Self::CallCreateTrapPrepareData,
		handler: &mut H,
	) -> Result<(Self::CallCreateTrapEnterData, GasedMachine<S, G>), ExitError>;
}

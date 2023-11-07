use crate::{Capture, ExitError, ExitResult, Machine};

pub trait Invoker<S, G, H, Tr> {
	type Interrupt;
	type CallCreateTrapData;

	fn feedback_trap_data(
		&self,
		result: ExitResult,
		child: Machine<S>,
		trap_data: Self::CallCreateTrapData,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		handler: &mut H,
	) -> Result<(), ExitError>;

	fn prepare_trap_data(
		&self,
		trap: Tr,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<Self::CallCreateTrapData, ExitError>, Self::Interrupt>;

	fn build_child_stack(
		&self,
		trap_data: &Self::CallCreateTrapData,
		handler: &mut H,
	) -> Result<(Machine<S>, G, bool), ExitError>;
}

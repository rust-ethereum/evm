use crate::{Capture, ExitError, ExitResult, GasedMachine};

pub trait Invoker<S, G, H, Tr> {
	type Interrupt;

	type TransactArgs;
	type TransactInvoke;
	type TransactValue;
	type SubstackInvoke;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<(Self::TransactInvoke, GasedMachine<S, G>), ExitError>;
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: GasedMachine<S, G>,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	fn exit_substack(
		&self,
		result: ExitResult,
		child: GasedMachine<S, G>,
		trap_data: Self::SubstackInvoke,
		parent: &mut GasedMachine<S, G>,
		handler: &mut H,
	) -> Result<(), ExitError>;

	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut GasedMachine<S, G>,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<(Self::SubstackInvoke, GasedMachine<S, G>), ExitError>, Self::Interrupt>;
}

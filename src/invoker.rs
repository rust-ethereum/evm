use crate::{Capture, ExitError, ExitResult};

pub trait Invoker<H, Tr> {
	type Machine;
	type Interrupt;

	type TransactArgs;
	type TransactInvoke;
	type TransactValue;
	type SubstackInvoke;

	fn run_machine(&self, machine: &mut Self::Machine, handler: &mut H) -> Capture<ExitResult, Tr>;
	fn step_machine(
		&self,
		machine: &mut Self::Machine,
		handler: &mut H,
	) -> Result<(), Capture<ExitResult, Tr>>;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<(Self::TransactInvoke, Self::Machine), ExitError>;

	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: Self::Machine,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	fn exit_substack(
		&self,
		result: ExitResult,
		child: Self::Machine,
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Machine,
		handler: &mut H,
	) -> Result<(), ExitError>;

	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut Self::Machine,
		handler: &mut H,
		depth: usize,
	) -> Capture<Result<(Self::SubstackInvoke, Self::Machine), ExitError>, Self::Interrupt>;
}

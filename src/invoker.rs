use crate::{Capture, ExitError, ExitResult};

pub enum InvokerControl<VE, VD> {
	Enter(VE),
	DirectExit(VD),
}

pub trait Invoker<H, Tr> {
	type Machine;
	type MachineDeconstruct;
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
	fn deconstruct_machine(&self, machine: Self::Machine) -> Self::MachineDeconstruct;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<Self::Machine, (ExitResult, Self::MachineDeconstruct)>,
		),
		ExitError,
	>;
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: Self::MachineDeconstruct,
		handler: &mut H,
	) -> Result<Self::TransactValue, ExitError>;

	fn enter_substack(
		&self,
		trap: Tr,
		machine: &mut Self::Machine,
		handler: &mut H,
		depth: usize,
	) -> Capture<
		Result<
			(
				Self::SubstackInvoke,
				InvokerControl<Self::Machine, (ExitResult, Self::MachineDeconstruct)>,
			),
			ExitError,
		>,
		Self::Interrupt,
	>;
	fn exit_substack(
		&self,
		result: ExitResult,
		child: Self::MachineDeconstruct,
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Machine,
		handler: &mut H,
	) -> Result<(), ExitError>;
}

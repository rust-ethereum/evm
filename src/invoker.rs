use crate::{Capture, ExitError, ExitResult};

pub enum InvokerControl<VE, VD> {
	Enter(VE),
	DirectExit(VD),
}

pub trait InvokerMachine<H, Tr> {
	type Deconstruct;

	fn step(&mut self, handler: &mut H) -> Result<(), Capture<ExitResult, Tr>>;
	fn run(&mut self, handler: &mut H) -> Capture<ExitResult, Tr>;
	fn deconstruct(self) -> Self::Deconstruct;
}

pub trait Invoker<H, Tr> {
	type Machine: InvokerMachine<H, Tr>;
	type Interrupt;

	type TransactArgs;
	type TransactInvoke;
	type TransactValue;
	type SubstackInvoke;

	fn new_transact(
		&self,
		args: Self::TransactArgs,
		handler: &mut H,
	) -> Result<
		(
			Self::TransactInvoke,
			InvokerControl<
				Self::Machine,
				(
					ExitResult,
					<Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
				),
			>,
		),
		ExitError,
	>;
	fn finalize_transact(
		&self,
		invoke: &Self::TransactInvoke,
		exit: ExitResult,
		machine: <Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
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
				InvokerControl<
					Self::Machine,
					(
						ExitResult,
						<Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
					),
				>,
			),
			ExitError,
		>,
		Self::Interrupt,
	>;
	fn exit_substack(
		&self,
		result: ExitResult,
		child: <Self::Machine as InvokerMachine<H, Tr>>::Deconstruct,
		trap_data: Self::SubstackInvoke,
		parent: &mut Self::Machine,
		handler: &mut H,
	) -> Result<(), ExitError>;
}

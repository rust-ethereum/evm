use crate::{
	Capture, Control, Etable, ExitError, ExitFatal, ExitResult, GasedMachine, Gasometer, Invoker,
	Machine, Opcode, RuntimeState,
};
use core::convert::Infallible;

struct Substack<S, G, TrD> {
	invoke: TrD,
	machine: GasedMachine<S, G>,
}

struct LastSubstack<S, G, Tr> {
	machine: GasedMachine<S, G>,
	status: LastSubstackStatus<Tr>,
}

enum LastSubstackStatus<Tr> {
	Running,
	ExternalTrapped,
	Exited(Capture<ExitResult, Tr>),
}

// Note: this should not be exposed to public because it does not implement
// Drop.
struct CallStack<'backend, 'invoker, S, G, H, Tr, I: Invoker<S, G, H, Tr>> {
	stack: Vec<Substack<S, G, I::SubstackInvoke>>,
	last: Option<LastSubstack<S, G, Tr>>,
	initial_depth: usize,
	backend: &'backend mut H,
	invoker: &'invoker I,
}

impl<'backend, 'invoker, S, G, H, Tr, I> CallStack<'backend, 'invoker, S, G, H, Tr, I>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	pub fn new(
		machine: GasedMachine<S, G>,
		initial_depth: usize,
		backend: &'backend mut H,
		invoker: &'invoker I,
	) -> Self {
		let call_stack = Self {
			stack: Vec::new(),
			last: Some(LastSubstack {
				machine,
				status: LastSubstackStatus::Running,
			}),
			initial_depth,
			backend,
			invoker,
		};

		call_stack
	}

	pub fn run<F>(
		&mut self,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<Result<(ExitResult, GasedMachine<S, G>), ExitFatal>, I::Interrupt>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			let step_ret = self.step(etable);

			if let Some(step_ret) = step_ret {
				return step_ret;
			}
		}
	}

	pub fn step<F>(
		&mut self,
		etable: &Etable<S, H, Tr, F>,
	) -> Option<Capture<Result<(ExitResult, GasedMachine<S, G>), ExitFatal>, I::Interrupt>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let mut step_ret = None;

		self.last = match self.last.take() {
			None => {
				step_ret = Some(Capture::Exit(Err(ExitFatal::AlreadyExited.into())));
				None
			}
			Some(LastSubstack {
				status: LastSubstackStatus::ExternalTrapped,
				machine,
			}) => Some(LastSubstack {
				status: LastSubstackStatus::Running,
				machine,
			}),
			Some(LastSubstack {
				status: LastSubstackStatus::Running,
				mut machine,
			}) => {
				let result = machine.run(self.backend, etable);
				Some(LastSubstack {
					status: LastSubstackStatus::Exited(result),
					machine,
				})
			}
			Some(LastSubstack {
				status: LastSubstackStatus::Exited(Capture::Exit(exit)),
				machine,
			}) => {
				if self.stack.is_empty() {
					step_ret = Some(Capture::Exit(Ok((exit, machine))));
					None
				} else {
					let mut upward = self
						.stack
						.pop()
						.expect("checked stack is not empty above; qed");

					let feedback_result = self.invoker.exit_substack(
						exit,
						machine,
						upward.invoke,
						&mut upward.machine,
						self.backend,
					);

					match feedback_result {
						Ok(()) => Some(LastSubstack {
							status: LastSubstackStatus::Running,
							machine: upward.machine,
						}),
						Err(err) => Some(LastSubstack {
							machine: upward.machine,
							status: LastSubstackStatus::Exited(Capture::Exit(Err(err))),
						}),
					}
				}
			}
			Some(LastSubstack {
				status: LastSubstackStatus::Exited(Capture::Trap(trap)),
				mut machine,
			}) => {
				match self.invoker.enter_substack(
					trap,
					&mut machine,
					self.backend,
					self.initial_depth + self.stack.len() + 1,
				) {
					Capture::Exit(Ok((trap_data, sub_machine))) => {
						self.stack.push(Substack {
							invoke: trap_data,
							machine,
						});

						Some(LastSubstack {
							status: LastSubstackStatus::Running,
							machine: sub_machine,
						})
					}
					Capture::Exit(Err(err)) => Some(LastSubstack {
						status: LastSubstackStatus::Exited(Capture::Exit(Err(err))),
						machine,
					}),
					Capture::Trap(trap) => {
						step_ret = Some(Capture::Trap(trap));

						Some(LastSubstack {
							status: LastSubstackStatus::ExternalTrapped,
							machine,
						})
					}
				}
			}
		};

		step_ret
	}
}

fn execute<S, G, H, Tr, I, F>(
	mut machine: GasedMachine<S, G>,
	initial_depth: usize,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> Result<(ExitResult, GasedMachine<S, G>), ExitFatal>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let mut result = machine.run(backend, etable);

	loop {
		match result {
			Capture::Exit(exit) => return Ok((exit, machine)),
			Capture::Trap(trap) => {
				match invoker.enter_substack(trap, &mut machine, backend, initial_depth + 1) {
					Capture::Exit(Ok((trap_data, sub_machine))) => {
						let (sub_result, sub_machine) = if heap_depth
							.map(|hd| initial_depth + 1 >= hd)
							.unwrap_or(false)
						{
							match CallStack::new(sub_machine, initial_depth + 1, backend, invoker)
								.run(etable)
							{
								Capture::Exit(v) => v?,
								Capture::Trap(infallible) => match infallible {},
							}
						} else {
							execute(
								sub_machine,
								initial_depth + 1,
								heap_depth,
								backend,
								invoker,
								etable,
							)?
						};

						match invoker.exit_substack(
							sub_result,
							sub_machine,
							trap_data,
							&mut machine,
							backend,
						) {
							Ok(()) => {
								result = machine.run(backend, etable);
							}
							Err(err) => return Ok((Err(err), machine)),
						}
					}
					Capture::Exit(Err(err)) => return Ok((Err(err), machine)),
					Capture::Trap(infallible) => match infallible {},
				}
			}
		}
	}
}

pub struct HeapTransact<
	'backend,
	'invoker,
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	H,
	Tr,
	I: Invoker<S, G, H, Tr>,
> {
	call_stack: CallStack<'backend, 'invoker, S, G, H, Tr, I>,
	transact_invoke: I::TransactInvoke,
}

impl<'backend, 'invoker, S, G, H, Tr, I> HeapTransact<'backend, 'invoker, S, G, H, Tr, I>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	pub fn new(
		args: I::TransactArgs,
		invoker: &'invoker I,
		backend: &'backend mut H,
	) -> Result<Self, ExitError> {
		let (transact_invoke, machine) = invoker.new_transact(args, backend)?;
		let call_stack = CallStack::new(machine, 0, backend, invoker);

		Ok(Self {
			transact_invoke,
			call_stack,
		})
	}

	pub fn step<F>(
		&mut self,
		etable: &Etable<S, H, Tr, F>,
	) -> Option<Capture<Result<I::TransactValue, ExitError>, I::Interrupt>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		match self.call_stack.step(etable) {
			None => None,
			Some(Capture::Trap(interrupt)) => Some(Capture::Trap(interrupt)),
			Some(Capture::Exit(Err(fatal))) => Some(Capture::Exit(Err(fatal.into()))),
			Some(Capture::Exit(Ok((ret, machine)))) => {
				Some(Capture::Exit(self.call_stack.invoker.finalize_transact(
					&self.transact_invoke,
					ret,
					machine,
					self.call_stack.backend,
				)))
			}
		}
	}

	pub fn run<F>(
		&mut self,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<Result<I::TransactValue, ExitError>, I::Interrupt>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			let step_ret = self.step(etable);

			if let Some(step_ret) = step_ret {
				return step_ret;
			}
		}
	}
}

impl<'backend, 'invoker, S, G, H, Tr, I> Drop for HeapTransact<'backend, 'invoker, S, G, H, Tr, I>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	fn drop(&mut self) {
		if let Some(mut last) = self.call_stack.last.take() {
			loop {
				if let Some(mut parent) = self.call_stack.stack.pop() {
					let _ = self.call_stack.invoker.exit_substack(
						ExitFatal::Unfinished.into(),
						last.machine,
						parent.invoke,
						&mut parent.machine,
						self.call_stack.backend,
					);

					last = LastSubstack {
						machine: parent.machine,
						status: LastSubstackStatus::Exited(Capture::Exit(
							ExitFatal::Unfinished.into(),
						)),
					};
				} else {
					break;
				}
			}

			let _ = self.call_stack.invoker.finalize_transact(
				&self.transact_invoke,
				ExitFatal::Unfinished.into(),
				last.machine,
				self.call_stack.backend,
			);
		}
	}
}

pub fn transact<S, G, H, Tr, I, F>(
	args: I::TransactArgs,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> Result<I::TransactValue, ExitError>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let (transact_invoke, machine) = invoker.new_transact(args, backend)?;
	let (ret, machine) = execute(machine, 0, heap_depth, backend, invoker, etable)?;
	invoker.finalize_transact(&transact_invoke, ret, machine, backend)
}

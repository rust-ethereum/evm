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

struct CallStack<'invoker, S, G, H, Tr, I: Invoker<S, G, H, Tr>> {
	stack: Vec<Substack<S, G, I::SubstackInvoke>>,
	last: Option<LastSubstack<S, G, Tr>>,
	initial_depth: usize,
	invoker: &'invoker I,
}

impl<'invoker, S, G, H, Tr, I> CallStack<'invoker, S, G, H, Tr, I>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	pub fn new(machine: GasedMachine<S, G>, initial_depth: usize, invoker: &'invoker I) -> Self {
		let call_stack = Self {
			stack: Vec::new(),
			last: Some(LastSubstack {
				machine,
				status: LastSubstackStatus::Running,
			}),
			initial_depth,
			invoker,
		};

		call_stack
	}

	pub fn run<F>(
		&mut self,
		backend: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<Result<(ExitResult, GasedMachine<S, G>), ExitFatal>, I::Interrupt>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			let step_ret = self.step(backend, etable);

			if let Some(step_ret) = step_ret {
				return step_ret;
			}
		}
	}

	pub fn step<F>(
		&mut self,
		backend: &mut H,
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
				let result = machine.run(backend, etable);
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
						backend,
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
					backend,
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
	backend: &mut H,
	initial_depth: usize,
	heap_depth: Option<usize>,
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
							match CallStack::new(sub_machine, initial_depth + 1, invoker)
								.run(backend, etable)
							{
								Capture::Exit(v) => v?,
								Capture::Trap(infallible) => match infallible {},
							}
						} else {
							execute(
								sub_machine,
								backend,
								initial_depth + 1,
								heap_depth,
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

pub struct HeapTransact<'invoker, S, G, H, Tr, I: Invoker<S, G, H, Tr>> {
	call_stack: Option<CallStack<'invoker, S, G, H, Tr, I>>,
	transact_invoke: I::TransactInvoke,
	invoker: &'invoker I,
	already_exited: bool,
}

impl<'invoker, S, G, H, Tr, I> HeapTransact<'invoker, S, G, H, Tr, I>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	pub fn new(invoke: I::TransactInvoke, invoker: &'invoker I) -> Self {
		Self {
			transact_invoke: invoke,
			invoker,
			call_stack: None,
			already_exited: false,
		}
	}

	pub fn step<F>(
		&mut self,
		backend: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Option<Capture<Result<I::TransactValue, ExitError>, I::Interrupt>>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		if self.already_exited {
			return Some(Capture::Exit(Err(ExitFatal::AlreadyExited.into())));
		}

		match self.call_stack {
			Some(ref mut call_stack) => match call_stack.step(backend, etable) {
				None => None,
				Some(Capture::Trap(interrupt)) => Some(Capture::Trap(interrupt)),
				Some(Capture::Exit(Err(fatal))) => {
					self.already_exited = true;
					Some(Capture::Exit(Err(fatal.into())))
				}
				Some(Capture::Exit(Ok((ret, machine)))) => {
					self.already_exited = true;
					Some(Capture::Exit(self.invoker.finalize_transact(
						&self.transact_invoke,
						ret,
						machine,
						backend,
					)))
				}
			},
			None => {
				let machine = match self.invoker.new_transact(&self.transact_invoke, backend) {
					Ok(machine) => machine,
					Err(err) => {
						self.already_exited = true;
						return Some(Capture::Exit(Err(err)));
					}
				};
				self.call_stack = Some(CallStack::new(machine, 0, self.invoker));
				None
			}
		}
	}

	pub fn run<F>(
		&mut self,
		backend: &mut H,
		etable: &Etable<S, H, Tr, F>,
	) -> Capture<Result<I::TransactValue, ExitError>, I::Interrupt>
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			let step_ret = self.step(backend, etable);

			if let Some(step_ret) = step_ret {
				return step_ret;
			}
		}
	}
}

pub fn transact<S, G, H, Tr, I, F>(
	invoke: I::TransactInvoke,
	backend: &mut H,
	heap_depth: Option<usize>,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> Result<I::TransactValue, ExitError>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let machine = invoker.new_transact(&invoke, backend)?;
	let (ret, machine) = execute(machine, backend, 0, heap_depth, invoker, etable)?;
	invoker.finalize_transact(&invoke, ret, machine, backend)
}

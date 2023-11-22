use crate::{Capture, ExitError, ExitFatal, ExitResult, Invoker, InvokerControl};
use core::convert::Infallible;

struct Substack<M, TrD> {
	invoke: TrD,
	machine: M,
}

struct LastSubstack<M, Tr> {
	machine: M,
	status: LastSubstackStatus<Tr>,
}

enum LastSubstackStatus<Tr> {
	Running,
	ExternalTrapped,
	Exited(Capture<ExitResult, Tr>),
}

// Note: this should not be exposed to public because it does not implement
// Drop.
struct CallStack<'backend, 'invoker, H, Tr, I: Invoker<H, Tr>> {
	stack: Vec<Substack<I::Machine, I::SubstackInvoke>>,
	last: Option<LastSubstack<I::Machine, Tr>>,
	initial_depth: usize,
	backend: &'backend mut H,
	invoker: &'invoker I,
}

impl<'backend, 'invoker, H, Tr, I> CallStack<'backend, 'invoker, H, Tr, I>
where
	I: Invoker<H, Tr>,
{
	pub fn new(
		machine: I::Machine,
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

	pub fn run(&mut self) -> Capture<Result<(ExitResult, I::Machine), ExitFatal>, I::Interrupt> {
		loop {
			let step_ret = self.step_run();

			if let Err(step_ret) = step_ret {
				return step_ret;
			}
		}
	}

	pub fn step(
		&mut self,
	) -> Result<(), Capture<Result<(ExitResult, I::Machine), ExitFatal>, I::Interrupt>> {
		self.step_with(|invoker, machine, handler| {
			let result = invoker.step_machine(machine, handler);
			match result {
				Ok(()) => LastSubstackStatus::Running,
				Err(result) => LastSubstackStatus::Exited(result),
			}
		})
	}

	pub fn step_run(
		&mut self,
	) -> Result<(), Capture<Result<(ExitResult, I::Machine), ExitFatal>, I::Interrupt>> {
		self.step_with(|invoker, machine, handler| {
			let result = invoker.run_machine(machine, handler);
			LastSubstackStatus::Exited(result)
		})
	}

	fn step_with<FS>(
		&mut self,
		fs: FS,
	) -> Result<(), Capture<Result<(ExitResult, I::Machine), ExitFatal>, I::Interrupt>>
	where
		FS: Fn(&I, &mut I::Machine, &mut H) -> LastSubstackStatus<Tr>,
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
				let status = fs(self.invoker, &mut machine, self.backend);
				Some(LastSubstack { status, machine })
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

					let machine = self.invoker.deconstruct_machine(machine);
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
					Capture::Exit(Ok((trap_data, InvokerControl::Enter(sub_machine)))) => {
						self.stack.push(Substack {
							invoke: trap_data,
							machine,
						});

						Some(LastSubstack {
							status: LastSubstackStatus::Running,
							machine: sub_machine,
						})
					}
					Capture::Exit(Ok((
						trap_data,
						InvokerControl::DirectExit((exit, sub_machine)),
					))) => {
						let feedback_result = self.invoker.exit_substack(
							exit,
							sub_machine,
							trap_data,
							&mut machine,
							self.backend,
						);

						match feedback_result {
							Ok(()) => Some(LastSubstack {
								status: LastSubstackStatus::Running,
								machine,
							}),
							Err(err) => Some(LastSubstack {
								machine,
								status: LastSubstackStatus::Exited(Capture::Exit(Err(err))),
							}),
						}
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

		match step_ret {
			Some(res) => Err(res),
			None => Ok(()),
		}
	}
}

fn execute<H, Tr, I>(
	mut machine: I::Machine,
	initial_depth: usize,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
) -> Result<(ExitResult, I::Machine), ExitFatal>
where
	I: Invoker<H, Tr, Interrupt = Infallible>,
{
	let mut result = invoker.run_machine(&mut machine, backend);

	loop {
		match result {
			Capture::Exit(exit) => return Ok((exit, machine)),
			Capture::Trap(trap) => {
				match invoker.enter_substack(trap, &mut machine, backend, initial_depth + 1) {
					Capture::Exit(Ok((trap_data, InvokerControl::Enter(sub_machine)))) => {
						let (sub_result, sub_machine) = if heap_depth
							.map(|hd| initial_depth + 1 >= hd)
							.unwrap_or(false)
						{
							match CallStack::new(sub_machine, initial_depth + 1, backend, invoker)
								.run()
							{
								Capture::Exit(v) => v?,
								Capture::Trap(infallible) => match infallible {},
							}
						} else {
							execute(sub_machine, initial_depth + 1, heap_depth, backend, invoker)?
						};

						match invoker.exit_substack(
							sub_result,
							invoker.deconstruct_machine(sub_machine),
							trap_data,
							&mut machine,
							backend,
						) {
							Ok(()) => {
								result = invoker.run_machine(&mut machine, backend);
							}
							Err(err) => return Ok((Err(err), machine)),
						}
					}
					Capture::Exit(Ok((
						trap_data,
						InvokerControl::DirectExit((sub_result, sub_machine)),
					))) => {
						match invoker.exit_substack(
							sub_result,
							sub_machine,
							trap_data,
							&mut machine,
							backend,
						) {
							Ok(()) => {
								result = invoker.run_machine(&mut machine, backend);
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

enum HeapTransactState<'backend, 'invoker, H, Tr, I: Invoker<H, Tr>> {
	Created {
		args: I::TransactArgs,
		invoker: &'invoker I,
		backend: &'backend mut H,
	},
	Running {
		call_stack: CallStack<'backend, 'invoker, H, Tr, I>,
		transact_invoke: I::TransactInvoke,
	},
}

pub struct HeapTransact<'backend, 'invoker, H, Tr, I: Invoker<H, Tr>>(
	Option<HeapTransactState<'backend, 'invoker, H, Tr, I>>,
);

impl<'backend, 'invoker, H, Tr, I> HeapTransact<'backend, 'invoker, H, Tr, I>
where
	I: Invoker<H, Tr>,
{
	pub fn new(
		args: I::TransactArgs,
		invoker: &'invoker I,
		backend: &'backend mut H,
	) -> Result<Self, ExitError> {
		Ok(Self(Some(HeapTransactState::Created {
			args,
			invoker,
			backend,
		})))
	}

	fn step_with<FS>(
		&mut self,
		fs: FS,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>>
	where
		FS: Fn(
			&mut CallStack<'backend, 'invoker, H, Tr, I>,
		) -> Result<(), Capture<Result<(ExitResult, I::Machine), ExitFatal>, I::Interrupt>>,
	{
		let ret;

		self.0 = match self.0.take() {
			Some(HeapTransactState::Running {
				mut call_stack,
				transact_invoke,
			}) => {
				ret = match fs(&mut call_stack) {
					Ok(()) => Ok(()),
					Err(Capture::Trap(interrupt)) => Err(Capture::Trap(interrupt)),
					Err(Capture::Exit(Err(fatal))) => Err(Capture::Exit(Err(fatal.into()))),
					Err(Capture::Exit(Ok((ret, machine)))) => {
						let machine = call_stack.invoker.deconstruct_machine(machine);
						Err(Capture::Exit(call_stack.invoker.finalize_transact(
							&transact_invoke,
							ret,
							machine,
							call_stack.backend,
						)))
					}
				};

				Some(HeapTransactState::Running {
					call_stack,
					transact_invoke,
				})
			}
			Some(HeapTransactState::Created {
				args,
				invoker,
				backend,
			}) => {
				let (transact_invoke, control) = match invoker.new_transact(args, backend) {
					Ok((transact_invoke, control)) => (transact_invoke, control),
					Err(err) => return Err(Capture::Exit(Err(err))),
				};

				match control {
					InvokerControl::Enter(machine) => {
						let call_stack = CallStack::new(machine, 0, backend, invoker);

						ret = Ok(());
						Some(HeapTransactState::Running {
							call_stack,
							transact_invoke,
						})
					}
					InvokerControl::DirectExit((exit, machine)) => {
						return Err(Capture::Exit(invoker.finalize_transact(
							&transact_invoke,
							exit,
							machine,
							backend,
						)));
					}
				}
			}
			None => return Err(Capture::Exit(Err(ExitFatal::AlreadyExited.into()))),
		};

		ret
	}

	pub fn step_run(
		&mut self,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>> {
		self.step_with(|call_stack| call_stack.step_run())
	}

	pub fn step(
		&mut self,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>> {
		self.step_with(|call_stack| call_stack.step())
	}

	pub fn run(&mut self) -> Capture<Result<I::TransactValue, ExitError>, I::Interrupt> {
		loop {
			let step_ret = self.step_run();

			if let Err(step_ret) = step_ret {
				return step_ret;
			}
		}
	}

	pub fn last_machine(&self) -> Option<&I::Machine> {
		match &self.0 {
			Some(HeapTransactState::Running { call_stack, .. }) => match &call_stack.last {
				Some(last) => Some(&last.machine),
				None => None,
			},
			_ => None,
		}
	}
}

impl<'backend, 'invoker, H, Tr, I> Drop for HeapTransact<'backend, 'invoker, H, Tr, I>
where
	I: Invoker<H, Tr>,
{
	fn drop(&mut self) {
		if let Some(state) = self.0.take() {
			match state {
				HeapTransactState::Running {
					mut call_stack,
					transact_invoke,
				} => {
					if let Some(mut last) = call_stack.last.take() {
						loop {
							if let Some(mut parent) = call_stack.stack.pop() {
								let last_machine =
									call_stack.invoker.deconstruct_machine(last.machine);
								let _ = call_stack.invoker.exit_substack(
									ExitFatal::Unfinished.into(),
									last_machine,
									parent.invoke,
									&mut parent.machine,
									call_stack.backend,
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

						let last_machine = call_stack.invoker.deconstruct_machine(last.machine);
						let _ = call_stack.invoker.finalize_transact(
							&transact_invoke,
							ExitFatal::Unfinished.into(),
							last_machine,
							call_stack.backend,
						);
					}
				}
				_ => (),
			}
		}
	}
}

pub fn transact<H, Tr, I>(
	args: I::TransactArgs,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
) -> Result<I::TransactValue, ExitError>
where
	I: Invoker<H, Tr, Interrupt = Infallible>,
{
	let (transact_invoke, control) = invoker.new_transact(args, backend)?;

	match control {
		InvokerControl::Enter(machine) => {
			let (ret, machine) = execute(machine, 0, heap_depth, backend, invoker)?;
			let machine = invoker.deconstruct_machine(machine);
			invoker.finalize_transact(&transact_invoke, ret, machine, backend)
		}
		InvokerControl::DirectExit((exit, machine)) => {
			invoker.finalize_transact(&transact_invoke, exit, machine, backend)
		}
	}
}

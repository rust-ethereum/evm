use alloc::vec::Vec;
use core::convert::Infallible;

use evm_interpreter::{Capture, ExitError, ExitFatal, ExitResult, Interpreter, StepInterpreter};

use crate::invoker::{Invoker, InvokerControl, InvokerExit};

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
#[allow(clippy::type_complexity)]
struct CallStack<'backend, 'invoker, H, I: Invoker<H>> {
	stack: Vec<Substack<I::Interpreter, I::SubstackInvoke>>,
	last: Option<LastSubstack<I::Interpreter, <I::Interpreter as Interpreter<H>>::Trap>>,
	initial_depth: usize,
	backend: &'backend mut H,
	invoker: &'invoker I,
}

impl<'backend, 'invoker, H, I> CallStack<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
{
	pub fn new(
		machine: I::Interpreter,
		initial_depth: usize,
		backend: &'backend mut H,
		invoker: &'invoker I,
	) -> Self {
		Self {
			stack: Vec::new(),
			last: Some(LastSubstack {
				machine,
				status: LastSubstackStatus::Running,
			}),
			initial_depth,
			backend,
			invoker,
		}
	}

	#[allow(clippy::type_complexity)]
	fn step_with<FS>(
		&mut self,
		fs: FS,
	) -> Result<(), Capture<Result<(ExitResult, I::Interpreter), ExitFatal>, I::Interrupt>>
	where
		FS: Fn(
			&mut I::Interpreter,
			&mut H,
		) -> LastSubstackStatus<<I::Interpreter as Interpreter<H>>::Trap>,
	{
		let mut step_ret = None;

		self.last = match self.last.take() {
			None => {
				step_ret = Some(Capture::Exit(Err(ExitFatal::AlreadyExited)));
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
				let status = fs(&mut machine, self.backend);
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

					let machine = machine.deconstruct();
					let exit = InvokerExit {
						result: exit,
						substate: Some(machine.0),
						retval: machine.1,
					};
					let feedback_result = self.invoker.exit_substack(
						upward.invoke,
						exit,
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
					*trap,
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
					Capture::Exit(Ok((trap_data, InvokerControl::DirectExit(exit)))) => {
						let feedback_result =
							self.invoker
								.exit_substack(trap_data, exit, &mut machine, self.backend);

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

impl<'backend, 'invoker, H, I> CallStack<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
	I::Interpreter: Interpreter<H>,
{
	#[allow(clippy::type_complexity)]
	pub fn run(
		&mut self,
	) -> Capture<Result<(ExitResult, I::Interpreter), ExitFatal>, I::Interrupt> {
		loop {
			let step_ret = self.step_run();

			if let Err(step_ret) = step_ret {
				return step_ret;
			}
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn step_run(
		&mut self,
	) -> Result<(), Capture<Result<(ExitResult, I::Interpreter), ExitFatal>, I::Interrupt>> {
		self.step_with(|machine, handler| {
			let result = machine.run(handler);
			LastSubstackStatus::Exited(result)
		})
	}
}

impl<'backend, 'invoker, H, I> CallStack<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
	I::Interpreter: StepInterpreter<H>,
{
	#[allow(clippy::type_complexity)]
	pub fn step(
		&mut self,
	) -> Result<(), Capture<Result<(ExitResult, I::Interpreter), ExitFatal>, I::Interrupt>> {
		self.step_with(|machine, handler| {
			let result = machine.step(handler);
			match result {
				Ok(()) => LastSubstackStatus::Running,
				Err(result) => LastSubstackStatus::Exited(result),
			}
		})
	}
}

fn execute<H, I>(
	mut machine: I::Interpreter,
	initial_depth: usize,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
) -> Result<(ExitResult, I::Interpreter), ExitFatal>
where
	I: Invoker<H, Interrupt = Infallible>,
	I::Interpreter: Interpreter<H>,
{
	let mut result = machine.run(backend);

	loop {
		match result {
			Capture::Exit(exit) => return Ok((exit, machine)),
			Capture::Trap(trap) => {
				match invoker.enter_substack(*trap, &mut machine, backend, initial_depth + 1) {
					Capture::Exit(Ok((trap_data, InvokerControl::Enter(sub_machine)))) => {
						let (sub_result, sub_machine) = if heap_depth
							.is_some_and(|hd| initial_depth + 1 >= hd)
						{
							match CallStack::new(sub_machine, initial_depth + 1, backend, invoker)
								.run()
							{
								Capture::Exit(v) => v?,
								Capture::Trap(infallible) => match *infallible {},
							}
						} else {
							execute(sub_machine, initial_depth + 1, heap_depth, backend, invoker)?
						};

						let sub_machine = sub_machine.deconstruct();
						let sub_exit = InvokerExit {
							result: sub_result,
							substate: Some(sub_machine.0),
							retval: sub_machine.1,
						};

						match invoker.exit_substack(trap_data, sub_exit, &mut machine, backend) {
							Ok(()) => {
								result = machine.run(backend);
							}
							Err(err) => return Ok((Err(err), machine)),
						}
					}
					Capture::Exit(Ok((trap_data, InvokerControl::DirectExit(sub_exit)))) => {
						match invoker.exit_substack(trap_data, sub_exit, &mut machine, backend) {
							Ok(()) => {
								result = machine.run(backend);
							}
							Err(err) => return Ok((Err(err), machine)),
						}
					}
					Capture::Exit(Err(err)) => return Ok((Err(err), machine)),
					Capture::Trap(infallible) => match *infallible {},
				}
			}
		}
	}
}

enum HeapTransactState<'backend, 'invoker, H, I: Invoker<H>> {
	Created {
		args: I::TransactArgs,
		invoker: &'invoker I,
		backend: &'backend mut H,
	},
	Running {
		call_stack: CallStack<'backend, 'invoker, H, I>,
		transact_invoke: I::TransactInvoke,
	},
}

/// Heap-based call stack for a transaction. This is suitable for single
/// stepping or debugging. The hybrid version [transact] uses a heap-based call
/// stack internally after certain depth.
pub struct HeapTransact<'backend, 'invoker, H, I: Invoker<H>>(
	Option<HeapTransactState<'backend, 'invoker, H, I>>,
);

impl<'backend, 'invoker, H, I> HeapTransact<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
{
	/// Create a new heap-based call stack.
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

	#[allow(clippy::type_complexity)]
	fn step_with<FS>(
		&mut self,
		fs: FS,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>>
	where
		FS: Fn(
			&mut CallStack<'backend, 'invoker, H, I>,
		) -> Result<
			(),
			Capture<Result<(ExitResult, I::Interpreter), ExitFatal>, I::Interrupt>,
		>,
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
						let machine = machine.deconstruct();
						let exit = InvokerExit {
							result: ret,
							substate: Some(machine.0),
							retval: machine.1,
						};
						Err(Capture::Exit(call_stack.invoker.finalize_transact(
							&transact_invoke,
							exit,
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
				let (transact_invoke, control) = invoker
					.new_transact(args, backend)
					.map_err(|err| Capture::Exit(Err(err)))?;

				match control {
					InvokerControl::Enter(machine) => {
						let call_stack = CallStack::new(machine, 0, backend, invoker);

						ret = Ok(());
						Some(HeapTransactState::Running {
							call_stack,
							transact_invoke,
						})
					}
					InvokerControl::DirectExit(exit) => {
						return Err(Capture::Exit(invoker.finalize_transact(
							&transact_invoke,
							exit,
							backend,
						)));
					}
				}
			}
			None => return Err(Capture::Exit(Err(ExitFatal::AlreadyExited.into()))),
		};

		ret
	}

	/// The machine of the last item on the call stack. This will be `None` if
	/// the heap stack is just created.
	pub fn last_interpreter(&self) -> Option<&I::Interpreter> {
		match &self.0 {
			Some(HeapTransactState::Running { call_stack, .. }) => match &call_stack.last {
				Some(last) => Some(&last.machine),
				None => None,
			},
			_ => None,
		}
	}
}

impl<'backend, 'invoker, H, I> HeapTransact<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
	I::Interpreter: Interpreter<H>,
{
	/// Step the call stack, but run the interpreter inside.
	#[allow(clippy::type_complexity)]
	pub fn step_run(
		&mut self,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>> {
		self.step_with(|call_stack| call_stack.step_run())
	}

	/// Run the call stack until it exits or receives interrupts.
	pub fn run(&mut self) -> Capture<Result<I::TransactValue, ExitError>, I::Interrupt> {
		loop {
			let step_ret = self.step_run();

			if let Err(step_ret) = step_ret {
				return step_ret;
			}
		}
	}
}

impl<'backend, 'invoker, H, I> HeapTransact<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
	I::Interpreter: StepInterpreter<H>,
{
	/// Step the call stack, and step the interpreter inside.
	#[allow(clippy::type_complexity)]
	pub fn step(
		&mut self,
	) -> Result<(), Capture<Result<I::TransactValue, ExitError>, I::Interrupt>> {
		self.step_with(|call_stack| call_stack.step())
	}
}

impl<'backend, 'invoker, H, I> Drop for HeapTransact<'backend, 'invoker, H, I>
where
	I: Invoker<H>,
{
	fn drop(&mut self) {
		if let Some(HeapTransactState::Running {
			mut call_stack,
			transact_invoke,
		}) = self.0.take()
		{
			if let Some(mut last) = call_stack.last.take() {
				while let Some(mut parent) = call_stack.stack.pop() {
					let last_machine = last.machine.deconstruct();
					let exit = InvokerExit {
						result: ExitFatal::Unfinished.into(),
						substate: Some(last_machine.0),
						retval: last_machine.1,
					};
					let _ = call_stack.invoker.exit_substack(
						parent.invoke,
						exit,
						&mut parent.machine,
						call_stack.backend,
					);

					last = LastSubstack {
						machine: parent.machine,
						status: LastSubstackStatus::Exited(Capture::Exit(
							ExitFatal::Unfinished.into(),
						)),
					};
				}

				let last_machine = last.machine.deconstruct();
				let exit = InvokerExit {
					result: ExitFatal::Unfinished.into(),
					substate: Some(last_machine.0),
					retval: last_machine.1,
				};
				let _ = call_stack.invoker.finalize_transact(
					&transact_invoke,
					exit,
					call_stack.backend,
				);
			}
		}
	}
}

/// Initiate a transaction, using a hybrid call stack.
///
/// Up until `heap_depth`, a stack-based call stack is used first. A stack-based
/// call stack is faster, but for really deep calls, it can reach the default
/// stack size limit of the platform and thus overflow.
///
/// After `heap_depth`, a heap-based call stack is then used.
///
/// If `heap_depth` is `None`, then always use a stack-based call stack.
///
/// Because a stack-based call stack cannot handle interrupts, the [Invoker]
/// type must have its `Interrupt` type set to [Infallible].
pub fn transact<H, I>(
	args: I::TransactArgs,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
) -> Result<I::TransactValue, ExitError>
where
	I: Invoker<H, Interrupt = Infallible>,
	I::Interpreter: Interpreter<H>,
{
	let (transact_invoke, control) = invoker.new_transact(args, backend)?;

	match control {
		InvokerControl::Enter(machine) => {
			let (ret, machine) = execute(machine, 0, heap_depth, backend, invoker)?;
			let machine = machine.deconstruct();
			let exit = InvokerExit {
				result: ret,
				substate: Some(machine.0),
				retval: machine.1,
			};
			invoker.finalize_transact(&transact_invoke, exit, backend)
		}
		InvokerControl::DirectExit(exit) => {
			invoker.finalize_transact(&transact_invoke, exit, backend)
		}
	}
}

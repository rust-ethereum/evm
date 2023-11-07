use crate::{
	Capture, Control, Etable, ExitError, ExitResult, GasedMachine, Gasometer, Invoker, Machine,
	Opcode, RuntimeState,
};
use core::convert::Infallible;

struct TrappedCallStackData<S, G, TrD> {
	trap_data: TrD,
	machine: GasedMachine<S, G>,
}

enum LastCallStackData<S, G, Tr> {
	Running {
		machine: GasedMachine<S, G>,
	},
	Exited {
		result: Capture<ExitResult, Tr>,
		machine: GasedMachine<S, G>,
	},
	ExternalTrapped {
		machine: GasedMachine<S, G>,
	},
}

pub struct CallStack<'backend, 'invoker, S, G, H, Tr, I: Invoker<S, G, H, Tr>> {
	stack: Vec<TrappedCallStackData<S, G, I::CallCreateTrapData>>,
	last: LastCallStackData<S, G, Tr>,
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
		backend: &'backend mut H,
		initial_depth: usize,
		invoker: &'invoker I,
	) -> Self {
		let last = LastCallStackData::Running { machine };

		let call_stack = Self {
			stack: Vec::new(),
			last,
			initial_depth,
			backend,
			invoker,
		};

		call_stack
	}

	/// Calling `expect_exit` after `execute` returns `Capture::Exit` is safe.
	pub fn expect_exit(self) -> (GasedMachine<S, G>, ExitResult) {
		match self.last {
			LastCallStackData::Exited {
				machine,
				result: Capture::Exit(exit),
			} => (machine, exit),
			_ => panic!("expected exit"),
		}
	}

	pub fn execute<F>(
		machine: GasedMachine<S, G>,
		backend: &'backend mut H,
		initial_depth: usize,
		invoker: &'invoker I,
		etable: &Etable<S, H, Tr, F>,
	) -> (Self, Capture<(), I::Interrupt>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let call_stack = Self::new(machine, backend, initial_depth, invoker);

		call_stack.run(etable)
	}

	pub fn run<F>(mut self, etable: &Etable<S, H, Tr, F>) -> (Self, Capture<(), I::Interrupt>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		loop {
			let step_ret;
			(self, step_ret) = self.step(etable);

			if let Some(step_ret) = step_ret {
				return (self, step_ret);
			}
		}
	}

	fn step<F>(mut self, etable: &Etable<S, H, Tr, F>) -> (Self, Option<Capture<(), I::Interrupt>>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let mut step_ret: Option<Capture<(), I::Interrupt>> = None;

		self.last = match self.last {
			LastCallStackData::ExternalTrapped { machine } => {
				LastCallStackData::Running { machine }
			}
			LastCallStackData::Running { mut machine } => {
				let result = machine.run(self.backend, etable);
				LastCallStackData::Exited { machine, result }
			}
			LastCallStackData::Exited {
				mut machine,
				result,
			} => match result {
				Capture::Exit(exit) => {
					if self.stack.is_empty() {
						step_ret = Some(Capture::Exit(()));

						LastCallStackData::Exited {
							machine,
							result: Capture::Exit(exit),
						}
					} else {
						let mut upward = self
							.stack
							.pop()
							.expect("checked stack is not empty above; qed");

						let feedback_result = self.invoker.exit_trap_stack(
							exit,
							machine,
							upward.trap_data,
							&mut upward.machine,
							self.backend,
						);

						match feedback_result {
							Ok(()) => LastCallStackData::Running {
								machine: upward.machine,
							},
							Err(err) => LastCallStackData::Exited {
								machine: upward.machine,
								result: Capture::Exit(Err(err)),
							},
						}
					}
				}
				Capture::Trap(trap) => {
					match self.invoker.prepare_trap(
						trap,
						&mut machine,
						self.backend,
						self.initial_depth + self.stack.len() + 1,
					) {
						Capture::Exit(Ok(trap_data)) => {
							match self.invoker.enter_trap_stack(&trap_data, self.backend) {
								Ok(sub_machine) => {
									self.stack.push(TrappedCallStackData { trap_data, machine });

									LastCallStackData::Running {
										machine: sub_machine,
									}
								}
								Err(err) => LastCallStackData::Exited {
									machine,
									result: Capture::Exit(Err(err)),
								},
							}
						}
						Capture::Exit(Err(err)) => LastCallStackData::Exited {
							machine,
							result: Capture::Exit(Err(err)),
						},
						Capture::Trap(trap) => {
							step_ret = Some(Capture::Trap(trap));

							LastCallStackData::ExternalTrapped { machine }
						}
					}
				}
			},
		};

		(self, step_ret)
	}
}

pub fn execute<S, G, H, Tr, I, F>(
	mut machine: GasedMachine<S, G>,
	backend: &mut H,
	initial_depth: usize,
	heap_depth: Option<usize>,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> (GasedMachine<S, G>, ExitResult)
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let mut result = machine.run(backend, etable);

	loop {
		match result {
			Capture::Exit(exit) => return (machine, exit),
			Capture::Trap(trap) => {
				let prepared_trap_data: Capture<
					Result<I::CallCreateTrapData, ExitError>,
					Infallible,
				> = invoker.prepare_trap(trap, &mut machine, backend, initial_depth + 1);

				match prepared_trap_data {
					Capture::Exit(Ok(trap_data)) => {
						match invoker.enter_trap_stack(&trap_data, backend) {
							Ok(sub_machine) => {
								let (sub_machine, sub_result) = if heap_depth
									.map(|hd| initial_depth + 1 >= hd)
									.unwrap_or(false)
								{
									let (call_stack, _infallible) = CallStack::execute(
										sub_machine,
										backend,
										initial_depth + 1,
										invoker,
										etable,
									);

									call_stack.expect_exit()
								} else {
									execute(
										sub_machine,
										backend,
										initial_depth + 1,
										heap_depth,
										invoker,
										etable,
									)
								};

								match invoker.exit_trap_stack(
									sub_result,
									sub_machine,
									trap_data,
									&mut machine,
									backend,
								) {
									Ok(()) => {
										result = machine.run(backend, etable);
									}
									Err(err) => return (machine, Err(err)),
								}
							}
							Err(err) => {
								return (machine, Err(err));
							}
						}
					}
					Capture::Exit(Err(err)) => return (machine, Err(err)),
					Capture::Trap(infallible) => match infallible {},
				}
			}
		}
	}
}

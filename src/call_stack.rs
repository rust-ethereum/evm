use crate::{
	gasometer::{run_with_gasometer, Gasometer, GasometerMergeStrategy},
	Capture, Control, Etable, ExitError, ExitResult, Invoker, Machine, Opcode,
	TransactionalBackend, TransactionalMergeStrategy,
};
use core::convert::Infallible;

struct TrappedCallStackData<S, TrD> {
	trap_data: TrD,
	machine: Machine<S>,
	is_static: bool,
}

enum LastCallStackData<S, Tr> {
	Running {
		machine: Machine<S>,
		is_static: bool,
	},
	Exited {
		result: Capture<ExitResult, Tr>,
		machine: Machine<S>,
		is_static: bool,
	},
	ExternalTrapped {
		machine: Machine<S>,
		is_static: bool,
	},
}

pub struct CallStack<'backend, 'invoker, S, G, H, Tr, I: Invoker<S, G, H, Tr>> {
	stack: Vec<TrappedCallStackData<S, I::CallCreateTrapData>>,
	last: LastCallStackData<S, Tr>,
	initial_depth: usize,
	backend: &'backend mut H,
	invoker: &'invoker I,
}

impl<'backend, 'invoker, S, G, H, Tr, I> CallStack<'backend, 'invoker, S, G, H, Tr, I>
where
	S: AsMut<G>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr>,
{
	pub fn new(
		machine: Machine<S>,
		backend: &'backend mut H,
		is_static: bool,
		initial_depth: usize,
		invoker: &'invoker I,
	) -> Self {
		let last = LastCallStackData::Running { machine, is_static };

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
	pub fn expect_exit(self) -> (Machine<S>, ExitResult) {
		match self.last {
			LastCallStackData::Exited {
				machine,
				result: Capture::Exit(exit),
				..
			} => (machine, exit),
			_ => panic!("expected exit"),
		}
	}

	pub fn execute<F>(
		machine: Machine<S>,
		backend: &'backend mut H,
		is_static: bool,
		initial_depth: usize,
		invoker: &'invoker I,
		etable: &Etable<S, H, Tr, F>,
	) -> (Self, Capture<(), I::Interrupt>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let call_stack = Self::new(machine, backend, is_static, initial_depth, invoker);

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
			LastCallStackData::ExternalTrapped { machine, is_static } => {
				LastCallStackData::Running { machine, is_static }
			}
			LastCallStackData::Running {
				mut machine,
				is_static,
			} => {
				let result = run_with_gasometer(&mut machine, self.backend, is_static, etable);
				LastCallStackData::Exited {
					machine,
					is_static,
					result,
				}
			}
			LastCallStackData::Exited {
				mut machine,
				is_static,
				result,
			} => match result {
				Capture::Exit(exit) => {
					if self.stack.is_empty() {
						step_ret = Some(Capture::Exit(()));

						LastCallStackData::Exited {
							machine,
							is_static,
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
								is_static: upward.is_static,
							},
							Err(err) => LastCallStackData::Exited {
								machine: upward.machine,
								is_static: upward.is_static,
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
								Ok((sub_machine, sub_gasometer, sub_is_static)) => {
									self.stack.push(TrappedCallStackData {
										trap_data,
										machine,
										is_static,
									});

									LastCallStackData::Running {
										machine: sub_machine,
										is_static: sub_is_static,
									}
								}
								Err(err) => LastCallStackData::Exited {
									machine,
									is_static,
									result: Capture::Exit(Err(err)),
								},
							}
						}
						Capture::Exit(Err(err)) => LastCallStackData::Exited {
							machine,
							is_static,
							result: Capture::Exit(Err(err)),
						},
						Capture::Trap(trap) => {
							step_ret = Some(Capture::Trap(trap));

							LastCallStackData::ExternalTrapped { machine, is_static }
						}
					}
				}
			},
		};

		(self, step_ret)
	}
}

pub fn execute<S, G, H, Tr, I, F>(
	mut machine: Machine<S>,
	backend: &mut H,
	is_static: bool,
	initial_depth: usize,
	heap_depth: Option<usize>,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> (Machine<S>, ExitResult)
where
	S: AsMut<G>,
	G: Gasometer<S, H>,
	I: Invoker<S, G, H, Tr, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let mut result = run_with_gasometer(&mut machine, backend, is_static, etable);

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
							Ok((sub_machine, sub_gasometer, sub_is_static)) => {
								let (sub_machine, sub_result) = if heap_depth
									.map(|hd| initial_depth + 1 >= hd)
									.unwrap_or(false)
								{
									let (call_stack, _infallible) = CallStack::execute(
										sub_machine,
										backend,
										sub_is_static,
										initial_depth + 1,
										invoker,
										etable,
									);

									call_stack.expect_exit()
								} else {
									execute(
										sub_machine,
										backend,
										sub_is_static,
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
										result = run_with_gasometer(
											&mut machine,
											backend,
											is_static,
											etable,
										);
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

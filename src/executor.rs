use crate::{
	gasometer::{run_with_gasometer, Gasometer, GasometerMergeStrategy},
	Capture, Control, Etable, ExitError, ExitResult, Machine, Opcode, TransactionalBackend,
	TransactionalMergeStrategy,
};
use core::convert::Infallible;

pub trait Invoker<S, H, Tr, G> {
	type Interrupt;
	type TrapData;

	fn feedback_trap_data(
		&self,
		result: ExitResult,
		child: Machine<S>,
		trap_data: Self::TrapData,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		handler: &mut H,
	) -> Result<(), ExitError>;

	fn prepare_trap_data(
		&self,
		trap: Tr,
		depth: usize,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		handler: &mut H,
	) -> Capture<Result<Self::TrapData, ExitError>, Self::Interrupt>;

	fn build_child_stack(
		&self,
		trap_data: &Self::TrapData,
		handler: &mut H,
	) -> Result<(Machine<S>, G, bool), ExitError>;
}

struct TrappedCallStackData<S, G, TrD> {
	trap_data: TrD,
	machine: Machine<S>,
	gasometer: G,
	is_static: bool,
}

enum LastCallStackData<S, Tr, G> {
	Running {
		machine: Machine<S>,
		gasometer: G,
		is_static: bool,
	},
	Exited {
		result: Capture<ExitResult, Tr>,
		machine: Machine<S>,
		gasometer: G,
		is_static: bool,
	},
	ExternalTrapped {
		machine: Machine<S>,
		gasometer: G,
		is_static: bool,
	},
}

pub struct CallStack<'backend, 'invoker, S, H, Tr, G, I: Invoker<S, H, Tr, G>> {
	stack: Vec<TrappedCallStackData<S, G, I::TrapData>>,
	last: LastCallStackData<S, Tr, G>,
	initial_depth: usize,
	backend: &'backend mut H,
	invoker: &'invoker I,
}

impl<'backend, 'invoker, S, H, Tr, G, I> CallStack<'backend, 'invoker, S, H, Tr, G, I>
where
	G: Gasometer<S, H>,
	H: TransactionalBackend,
	I: Invoker<S, H, Tr, G>,
{
	pub fn new(
		machine: Machine<S>,
		gasometer: G,
		is_static: bool,
		initial_depth: usize,
		backend: &'backend mut H,
		invoker: &'invoker I,
	) -> Self {
		let last = LastCallStackData::Running {
			machine,
			gasometer,
			is_static,
		};

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
	pub fn expect_exit(self) -> (Machine<S>, G, ExitResult) {
		match self.last {
			LastCallStackData::Exited {
				machine,
				gasometer,
				result: Capture::Exit(exit),
				..
			} => (machine, gasometer, exit),
			_ => panic!("expected exit"),
		}
	}

	pub fn execute<F>(
		machine: Machine<S>,
		gasometer: G,
		is_static: bool,
		initial_depth: usize,
		backend: &'backend mut H,
		invoker: &'invoker I,
		etable: &Etable<S, H, Tr, F>,
	) -> (Self, Capture<(), I::Interrupt>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let call_stack = Self::new(
			machine,
			gasometer,
			is_static,
			initial_depth,
			backend,
			invoker,
		);

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
			LastCallStackData::ExternalTrapped {
				machine,
				gasometer,
				is_static,
			} => LastCallStackData::Running {
				machine,
				gasometer,
				is_static,
			},
			LastCallStackData::Running {
				mut machine,
				mut gasometer,
				is_static,
			} => {
				let result = run_with_gasometer(
					&mut machine,
					&mut gasometer,
					is_static,
					self.backend,
					etable,
				);
				LastCallStackData::Exited {
					machine,
					gasometer,
					is_static,
					result,
				}
			}
			LastCallStackData::Exited {
				mut machine,
				mut gasometer,
				is_static,
				result,
			} => match result {
				Capture::Exit(exit) => {
					if self.stack.is_empty() {
						step_ret = Some(Capture::Exit(()));

						LastCallStackData::Exited {
							machine,
							gasometer,
							is_static,
							result: Capture::Exit(exit),
						}
					} else {
						let mut upward = self
							.stack
							.pop()
							.expect("checked stack is not empty above; qed");

						match &exit {
							Ok(_) => {
								self.backend
									.pop_substate(TransactionalMergeStrategy::Commit);
								upward
									.gasometer
									.merge(gasometer, GasometerMergeStrategy::Commit);
							}
							Err(ExitError::Reverted) => {
								self.backend
									.pop_substate(TransactionalMergeStrategy::Discard);
								upward
									.gasometer
									.merge(gasometer, GasometerMergeStrategy::Revert);
							}
							Err(_) => {
								self.backend
									.pop_substate(TransactionalMergeStrategy::Discard);
							}
						};

						let feedback_result = self.invoker.feedback_trap_data(
							exit,
							machine,
							upward.trap_data,
							&mut upward.machine,
							&mut upward.gasometer,
							self.backend,
						);

						match feedback_result {
							Ok(()) => LastCallStackData::Running {
								machine: upward.machine,
								gasometer: upward.gasometer,
								is_static: upward.is_static,
							},
							Err(err) => LastCallStackData::Exited {
								machine: upward.machine,
								gasometer: upward.gasometer,
								is_static: upward.is_static,
								result: Capture::Exit(Err(err)),
							},
						}
					}
				}
				Capture::Trap(trap) => {
					match self.invoker.prepare_trap_data(
						trap,
						self.initial_depth + self.stack.len() + 1,
						&mut machine,
						&mut gasometer,
						self.backend,
					) {
						Capture::Exit(Ok(trap_data)) => {
							self.backend.push_substate();

							match self.invoker.build_child_stack(&trap_data, self.backend) {
								Ok((sub_machine, sub_gasometer, sub_is_static)) => {
									self.stack.push(TrappedCallStackData {
										trap_data,
										machine,
										gasometer,
										is_static,
									});

									LastCallStackData::Running {
										machine: sub_machine,
										gasometer: sub_gasometer,
										is_static: sub_is_static,
									}
								}
								Err(err) => {
									self.backend
										.pop_substate(TransactionalMergeStrategy::Discard);

									LastCallStackData::Exited {
										machine,
										gasometer,
										is_static,
										result: Capture::Exit(Err(err)),
									}
								}
							}
						}
						Capture::Exit(Err(err)) => LastCallStackData::Exited {
							machine,
							gasometer,
							is_static,
							result: Capture::Exit(Err(err)),
						},
						Capture::Trap(trap) => {
							step_ret = Some(Capture::Trap(trap));

							LastCallStackData::ExternalTrapped {
								machine,
								gasometer,
								is_static,
							}
						}
					}
				}
			},
		};

		(self, step_ret)
	}
}

pub fn execute<S, H, Tr, G, I, F>(
	mut machine: Machine<S>,
	mut gasometer: G,
	is_static: bool,
	initial_depth: usize,
	heap_depth: Option<usize>,
	backend: &mut H,
	invoker: &I,
	etable: &Etable<S, H, Tr, F>,
) -> (Machine<S>, G, ExitResult)
where
	G: Gasometer<S, H>,
	H: TransactionalBackend,
	I: Invoker<S, H, Tr, G, Interrupt = Infallible>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	let mut result = run_with_gasometer(&mut machine, &mut gasometer, is_static, backend, etable);

	loop {
		match result {
			Capture::Exit(exit) => return (machine, gasometer, exit),
			Capture::Trap(trap) => {
				let prepared_trap_data: Capture<Result<I::TrapData, ExitError>, Infallible> =
					invoker.prepare_trap_data(
						trap,
						initial_depth + 1,
						&mut machine,
						&mut gasometer,
						backend,
					);

				match prepared_trap_data {
					Capture::Exit(Ok(trap_data)) => {
						backend.push_substate();

						match invoker.build_child_stack(&trap_data, backend) {
							Ok((sub_machine, sub_gasometer, sub_is_static)) => {
								let (sub_machine, sub_gasometer, sub_result) = if heap_depth
									.map(|hd| initial_depth + 1 >= hd)
									.unwrap_or(false)
								{
									let (call_stack, _infallible) = CallStack::execute(
										sub_machine,
										sub_gasometer,
										sub_is_static,
										initial_depth + 1,
										backend,
										invoker,
										etable,
									);

									call_stack.expect_exit()
								} else {
									execute(
										sub_machine,
										sub_gasometer,
										sub_is_static,
										initial_depth + 1,
										heap_depth,
										backend,
										invoker,
										etable,
									)
								};

								match sub_result {
									Ok(_) => {
										backend.pop_substate(TransactionalMergeStrategy::Commit);
										gasometer
											.merge(sub_gasometer, GasometerMergeStrategy::Commit);
									}
									Err(ExitError::Reverted) => {
										backend.pop_substate(TransactionalMergeStrategy::Discard);
										gasometer
											.merge(sub_gasometer, GasometerMergeStrategy::Revert);
									}
									Err(_) => {
										backend.pop_substate(TransactionalMergeStrategy::Discard);
									}
								}

								match invoker.feedback_trap_data(
									sub_result,
									sub_machine,
									trap_data,
									&mut machine,
									&mut gasometer,
									backend,
								) {
									Ok(()) => {
										result = run_with_gasometer(
											&mut machine,
											&mut gasometer,
											is_static,
											backend,
											etable,
										);
									}
									Err(err) => return (machine, gasometer, Err(err)),
								}
							}
							Err(err) => {
								backend.pop_substate(TransactionalMergeStrategy::Discard);

								return (machine, gasometer, Err(err));
							}
						}
					}
					Capture::Exit(Err(err)) => return (machine, gasometer, Err(err)),
					Capture::Trap(infallible) => match infallible {},
				}
			}
		}
	}
}

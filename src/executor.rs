use crate::{
	gasometer::{run_with_gasometer, Gasometer, GasometerMergeStrategy},
	Capture, Control, Etable, ExitError, ExitResult, Machine, Opcode, TransactionalBackend,
	TransactionalMergeStrategy,
};

pub trait Invoker<S, H, Tr, G> {
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
		machine: &mut Machine<S>,
		gasometer: &mut G,
		handler: &mut H,
	) -> Capture<Result<Self::TrapData, ExitError>, Tr>;

	fn build_child_stack(
		&self,
		trap_data: Self::TrapData,
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
			backend,
			invoker,
		};

		call_stack
	}

	pub fn run<F>(mut self, etable: &Etable<S, H, Tr, F>) -> (Self, Capture<(), Tr>)
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

	fn step<F>(mut self, etable: &Etable<S, H, Tr, F>) -> (Self, Option<Capture<(), Tr>>)
	where
		F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
	{
		let mut step_ret: Option<Capture<(), Tr>> = None;

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
					self.backend,
					is_static,
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
								upward
									.gasometer
									.merge(gasometer, GasometerMergeStrategy::Discard);
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
						&mut machine,
						&mut gasometer,
						self.backend,
					) {
						Capture::Exit(Ok(trap_data)) => {
							self.backend.push_substate();

							match self.invoker.build_child_stack(trap_data, self.backend) {
								Ok((machine, gasometer, is_static)) => LastCallStackData::Running {
									machine,
									gasometer,
									is_static,
								},
								Err(err) => LastCallStackData::Exited {
									machine,
									gasometer,
									is_static,
									result: Capture::Exit(Err(err)),
								},
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

use crate::{
	gasometer::{run_with_gasometer, ExecutionResult, Gasometer, StandardGasometer},
	Capture, Config, RuntimeState, RuntimeTrap, StandardMachine, StandardTrap,
	TransactionalBackend,
};

struct CallStackData<'config> {
	trap: StandardTrap,
	gasometer: StandardGasometer<'config>,
}

pub struct Executor<'config, B> {
	config: &'config Config,
	backend: B,
	call_stack: Vec<CallStackData<'config>>,
}

impl<'config, B> Executor<'config, B>
where
	B: TransactionalBackend,
{
	pub fn execute(
		&mut self,
		machine: StandardMachine,
		gasometer: StandardGasometer<'config>,
	) -> ExecutionResult<RuntimeState, StandardGasometer> {
		let initial_stack_depth = self.call_stack.len();

		self.backend.push_substate();
		let mut result = run_with_gasometer(
			machine,
			gasometer,
			unimplemented!(),
			unimplemented!(),
			unimplemented!(),
		);

		loop {
			match result {
				Capture::Exit(r) => {
					self.backend.pop_substate(unimplemented!());

					if self.call_stack.len() == initial_stack_depth || self.call_stack.len() == 0 {
						return r;
					} else {
						let stack_data = self
							.call_stack
							.pop()
							.expect("checked above call stack is not empty; qed");
						stack_data.gasometer.merge(gasometer, unimplemented!());
						let gasometer = stack_data.gasometer;

						match stack_data.trap {
							RuntimeTrap::Call(call_trap) => {
								let (machine, feedback_res) =
									call_trap.feedback(unimplemented!(), unimplemented!());

								match feedback_res {
									Ok(()) => {
										result = run_with_gasometer(
											machine,
											gasometer,
											unimplemented!(),
											unimplemented!(),
											unimplemented!(),
										);
									}
									Err(e) => {
										result = Capture::Exit(ExecutionResult::ErrLeftGas(
											machine, gasometer, e,
										));
									}
								}
							}
							RuntimeTrap::Create(create_trap) => unimplemented!(),
						}
					}
				}
				Capture::Trap((trap, gasometer)) => {
					self.backend.push_substate();

					let stack_data = CallStackData { trap, gasometer };

					let sub_machine: StandardMachine = unimplemented!();
					let sub_gasometer: StandardGasometer<'config> = unimplemented!();

					self.call_stack.push(stack_data);
					result = run_with_gasometer(
						sub_machine,
						sub_gasometer,
						unimplemented!(),
						unimplemented!(),
						unimplemented!(),
					);
				}
			}
		}
	}
}

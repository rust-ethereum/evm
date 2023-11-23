use crate::{
	Capture, Control, Etable, ExitResult, Gasometer, InvokerMachine, Machine, Opcode, RuntimeState,
};

pub struct ColoredMachine<S, G, C> {
	pub machine: Machine<S>,
	pub gasometer: G,
	pub is_static: bool,
	pub color: C,
}

impl<S, G, H, C, Tr> InvokerMachine<H, Tr> for ColoredMachine<S, G, C>
where
	C: Color<S, G, H, Tr>,
{
	type Deconstruct = (S, G, Vec<u8>);

	fn step(&mut self, handler: &mut H) -> Result<(), Capture<ExitResult, Tr>> {
		self.color.step(
			&mut self.machine,
			&mut self.gasometer,
			self.is_static,
			handler,
		)
	}

	fn run(&mut self, handler: &mut H) -> Capture<ExitResult, Tr> {
		self.color.run(
			&mut self.machine,
			&mut self.gasometer,
			self.is_static,
			handler,
		)
	}

	fn deconstruct(self) -> Self::Deconstruct {
		(self.machine.state, self.gasometer, self.machine.retval)
	}
}

pub trait Color<S, G, H, Tr> {
	fn step(
		&self,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		is_static: bool,
		handler: &mut H,
	) -> Result<(), Capture<ExitResult, Tr>>;

	fn run(
		&self,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		is_static: bool,
		handler: &mut H,
	) -> Capture<ExitResult, Tr>;
}

impl<'etable, S, G, H, Tr, F> Color<S, G, H, Tr> for &'etable Etable<S, H, Tr, F>
where
	S: AsMut<RuntimeState>,
	G: Gasometer<S, H>,
	F: Fn(&mut Machine<S>, &mut H, Opcode, usize) -> Control<Tr>,
{
	fn step(
		&self,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		is_static: bool,
		handler: &mut H,
	) -> Result<(), Capture<ExitResult, Tr>> {
		match gasometer.record_step(&machine, is_static, handler) {
			Ok(()) => {
				machine.state.as_mut().gas = gasometer.gas().into();
				machine.step(handler, self)
			}
			Err(e) => return Err(Capture::Exit(Err(e))),
		}
	}

	fn run(
		&self,
		machine: &mut Machine<S>,
		gasometer: &mut G,
		is_static: bool,
		handler: &mut H,
	) -> Capture<ExitResult, Tr> {
		loop {
			match gasometer.record_stepn(&machine, is_static, handler) {
				Ok(stepn) => {
					machine.state.as_mut().gas = gasometer.gas().into();
					match machine.stepn(stepn, handler, self) {
						Ok(()) => (),
						Err(c) => return c,
					}
				}
				Err(e) => return Capture::Exit(Err(e)),
			}
		}
	}
}

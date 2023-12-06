mod etable;

pub use self::etable::EtableInterpreter;

use crate::{Capture, ExitResult, Machine};
use alloc::vec::Vec;

pub type StateFor<H, I> = <I as Interpreter<H>>::State;
pub type TrapFor<H, I> = <I as Interpreter<H>>::Trap;
pub type DeconstructFor<H, I> = (StateFor<H, I>, Vec<u8>);

pub trait Interpreter<H> {
	type State;
	type Trap;

	fn machine(&self) -> &Machine<Self::State>;
	fn machine_mut(&mut self) -> &mut Machine<Self::State>;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap>;
	fn advance(&mut self);
}

pub trait StepInterpreter<H>: Interpreter<H> {
	fn step(&mut self, handle: &mut H) -> Result<(), Capture<ExitResult, Self::Trap>>;
}

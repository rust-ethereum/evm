mod etable;

pub use self::etable::EtableInterpreter;

use crate::{Capture, ExitResult, Machine};
use alloc::vec::Vec;

pub type StateFor<I> = <I as Interpreter>::State;
pub type TrapFor<I> = <I as Interpreter>::Trap;
pub type HandleFor<I> = <I as Interpreter>::Handle;
pub type DeconstructFor<I> = (StateFor<I>, Vec<u8>);

pub trait Interpreter {
	type State;
	type Handle;
	type Trap;

	fn machine(&self) -> &Machine<Self::State>;
	fn machine_mut(&mut self) -> &mut Machine<Self::State>;

	fn deconstruct(self) -> (Self::State, Vec<u8>);
	fn run(&mut self, handle: &mut Self::Handle) -> Capture<ExitResult, Self::Trap>;
	fn advance(&mut self);
}

pub trait StepInterpreter: Interpreter {
	fn step(&mut self, handle: &mut Self::Handle) -> Result<(), Capture<ExitResult, Self::Trap>>;
}

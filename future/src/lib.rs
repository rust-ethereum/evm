extern crate alloc;

use core::{future::Future, pin::Pin, convert::Infallible};
use alloc::{task::Wake, sync::Arc};
use evm::{interpreter::{Interpreter, FeedbackInterpreter, error::{ExitResult, Capture}}};
use environmental::environmental;

pub struct FutureInterpreterControl<'interpreter, 'handle, S, H> {
	state: &'interpreter mut S,
	retbuf: &'interpreter mut Vec<u8>,
	handle: &'handle mut H,
}

impl<'a, S, H> Wake for FutureInterpreterControl<'a, S, H> {
	fn wake(self: Arc<Self>) { }
}

pub struct FutureInterpreter<S> {
	state: S,
	retbuf: Vec<u8>,
	inner: Pin<Box<dyn Future<Output=ExitResult>>>,
}

impl<S, H> Interpreter<H> for FutureInterpreter<S> {
	type State = S;
	type Trap = Infallible;

	fn deconstruct(self) -> (S, Vec<u8>) {
		(self.state, self.retbuf)
	}

	fn run<'interpreter, 'handle>(&'interpreter mut self, handle: &'handle mut H) -> Capture<ExitResult, Self::Trap> {
		environmental!(control: FutureInterpreterControl<'interpreter, 'handle, S, H>);

		let control0 = FutureInterpreterControl {
			state: &mut self.state,
			retbuf: &mut self.retbuf,
			handle,
		};

		todo!()
	}
}

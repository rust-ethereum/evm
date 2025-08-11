#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{
	cell::Cell,
	future::Future,
	marker::PhantomData,
	pin::Pin,
	task::{Context, Poll, Waker},
};
use evm::interpreter::{
	Capture, ExitError, ExitFatal, ExitResult, FeedbackInterpreter, Interpreter,
};

pub trait FutureInterpreterAction<S, H> {
	type Feedback;
	type Trap;

	fn run(
		self,
		state: &mut S,
		retbuf: &mut Vec<u8>,
		handle: &mut H,
	) -> Capture<Self::Feedback, Self::Trap>;
}

pub struct FutureInterpreterSubmit<A, F> {
	action: Cell<Option<A>>,
	action_feedback: Cell<Option<F>>,
}

impl<A, F> FutureInterpreterSubmit<A, F> {
	fn new() -> Self {
		Self {
			action: Cell::new(None),
			action_feedback: Cell::new(None),
		}
	}

	pub fn submit(self: Rc<Self>, action: A) -> impl Future<Output = F> {
		struct SubmitFuture<A, F>(Cell<Option<A>>, Rc<FutureInterpreterSubmit<A, F>>);

		impl<A, F> Future for SubmitFuture<A, F> {
			type Output = F;

			fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<F> {
				let action_feedback = self.1.action_feedback.take();
				if let Some(action_feedback) = action_feedback {
					Poll::Ready(action_feedback)
				} else {
					let action = match self.0.replace(None) {
						Some(action) => action,
						None => panic!("future was already finished; should not be polled"),
					};
					self.1.action.set(Some(action));
					Poll::Pending
				}
			}
		}

		SubmitFuture(Cell::new(Some(action)), self.clone())
	}
}

pub struct FutureInterpreter<A, F, S, Tr> {
	state: S,
	retbuf: Vec<u8>,
	inner: Pin<Box<dyn Future<Output = ExitResult>>>,
	submit: Rc<FutureInterpreterSubmit<A, F>>,
	_marker: PhantomData<Tr>,
}

impl<A, F, S, Tr> FutureInterpreter<A, F, S, Tr> {
	pub fn new<Fn, Fu>(state: S, retbuf: Vec<u8>, f: Fn) -> Self
	where
		Fn: FnOnce(Rc<FutureInterpreterSubmit<A, F>>) -> Fu,
		Fu: Future<Output = ExitResult> + 'static,
	{
		let submit = Rc::new(FutureInterpreterSubmit::new());
		Self {
			state,
			retbuf,
			inner: Box::pin(f(submit.clone())),
			submit,
			_marker: PhantomData,
		}
	}
}

impl<A, F, S, H, Tr> Interpreter<H> for FutureInterpreter<A, F, S, Tr>
where
	F: 'static,
	A: FutureInterpreterAction<S, H, Feedback = F> + 'static,
	Tr: From<A::Trap>,
{
	type State = S;
	type Trap = Tr;

	fn deconstruct(self) -> (S, Vec<u8>) {
		(self.state, self.retbuf)
	}

	fn state(&self) -> &S {
		&self.state
	}

	fn state_mut(&mut self) -> &mut S {
		&mut self.state
	}

	fn run(&mut self, handle: &mut H) -> Capture<ExitResult, Self::Trap> {
		let waker = Waker::noop();
		let mut ctx = Context::from_waker(waker);

		loop {
			match self.inner.as_mut().poll(&mut ctx) {
				Poll::Ready(ret) => return Capture::Exit(ret),
				Poll::Pending => {
					let action = match self.submit.action.replace(None) {
						Some(action) => action,
						None => {
							return Capture::Exit(
								ExitFatal::Other("cannot advance future".into()).into(),
							)
						}
					};

					match action.run(&mut self.state, &mut self.retbuf, handle) {
						Capture::Exit(feedback) => {
							self.submit.action_feedback.set(Some(feedback));
						}
						Capture::Trap(trap) => return Capture::Trap(Box::new((*trap).into())),
					}
				}
			}
		}
	}
}

impl<Feedback, A, F, S, H, Tr> FeedbackInterpreter<H, Feedback> for FutureInterpreter<A, F, S, Tr>
where
	F: 'static,
	A: FutureInterpreterAction<S, H, Feedback = F> + 'static,
	Tr: From<A::Trap>,
	Feedback: Into<A::Feedback>,
{
	fn feedback(&mut self, feedback: Feedback, _handler: &mut H) -> Result<(), ExitError> {
		let feedback: A::Feedback = feedback.into();
		self.submit.action_feedback.set(Some(feedback));
		Ok(())
	}
}

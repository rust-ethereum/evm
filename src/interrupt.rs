use crate::{Runtime, Handler};

pub enum Resolve<'context, H: Handler> {
	Create(ResolveCreate<'context, H::CreateInterrupt>),
	Call(ResolveCall<'context, H::CallInterrupt>),
}

pub struct ResolveCreate<'context, Interrupt> {
	runtime: Runtime<'context>,
	interrupt: Interrupt,
}

impl<'context, Interrupt> ResolveCreate<'context, Interrupt> {
	pub(crate) fn new(runtime: Runtime<'context>, interrupt: Interrupt) -> Self {
		Self { runtime, interrupt }
	}
}

pub struct ResolveCall<'context, Interrupt> {
	runtime: Runtime<'context>,
	interrupt: Interrupt,
}

impl<'context, Interrupt> ResolveCall<'context, Interrupt> {
	pub(crate) fn new(runtime: Runtime<'context>, interrupt: Interrupt) -> Self {
		Self { runtime, interrupt }
	}
}

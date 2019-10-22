use crate::{Runtime, Handler};

pub enum Resolve<H: Handler> {
	Create(H::CreateInterrupt, ResolveCreate),
	Call(H::CallInterrupt, ResolveCall),
}

pub struct ResolveCreate {
	runtime: Runtime,
}

impl ResolveCreate {
	pub(crate) fn new(runtime: Runtime) -> Self {
		Self { runtime }
	}
}

pub struct ResolveCall {
	runtime: Runtime,
}

impl ResolveCall {
	pub(crate) fn new(runtime: Runtime) -> Self {
		Self { runtime }
	}
}

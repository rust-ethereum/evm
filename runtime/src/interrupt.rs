use crate::{Runtime, Handler, ExitError};

pub enum Resolve<'a, H: Handler> {
	Create(H::CreateInterrupt, ResolveCreate<'a>),
	Call(H::CallInterrupt, ResolveCall<'a>),
}

pub struct ResolveCreate<'a> {
	runtime: &'a mut Runtime,
}

impl<'a> ResolveCreate<'a> {
	pub(crate) fn new(runtime: &'a mut Runtime) -> Self {
		Self { runtime }
	}
}

impl<'a> Drop for ResolveCreate<'a> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitError::Other("create interrupt dropped").into());
		self.runtime.machine.exit(ExitError::Other("create interrupt dropped").into());
	}
}

pub struct ResolveCall<'a> {
	runtime: &'a mut Runtime,
}

impl<'a> ResolveCall<'a> {
	pub(crate) fn new(runtime: &'a mut Runtime) -> Self {
		Self { runtime }
	}
}

impl<'a> Drop for ResolveCall<'a> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitError::Other("call interrupt dropped").into());
		self.runtime.machine.exit(ExitError::Other("call interrupt dropped").into());
	}
}

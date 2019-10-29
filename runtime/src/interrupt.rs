use crate::{Runtime, Handler, ExitError};

pub enum Resolve<'a, 'config, H: Handler> {
	Create(H::CreateInterrupt, ResolveCreate<'a, 'config>),
	Call(H::CallInterrupt, ResolveCall<'a, 'config>),
}

pub struct ResolveCreate<'a, 'config> {
	runtime: &'a mut Runtime<'config>,
}

impl<'a, 'config> ResolveCreate<'a, 'config> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config>) -> Self {
		Self { runtime }
	}
}

impl<'a, 'config> Drop for ResolveCreate<'a, 'config> {
	fn drop(&mut self) {
		self.runtime.status = Err(Err(ExitError::Other("create interrupt dropped")));
		self.runtime.machine.exit(Err(ExitError::Other("create interrupt dropped")));
	}
}

pub struct ResolveCall<'a, 'config> {
	runtime: &'a mut Runtime<'config>,
}

impl<'a, 'config> ResolveCall<'a, 'config> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config>) -> Self {
		Self { runtime }
	}
}

impl<'a, 'config> Drop for ResolveCall<'a, 'config> {
	fn drop(&mut self) {
		self.runtime.status = Err(Err(ExitError::Other("call interrupt dropped")));
		self.runtime.machine.exit(Err(ExitError::Other("call interrupt dropped")));
	}
}

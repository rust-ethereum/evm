use crate::{Runtime, Handler};

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

pub struct ResolveCall<'a> {
	runtime: &'a mut Runtime,
}

impl<'a> ResolveCall<'a> {
	pub(crate) fn new(runtime: &'a mut Runtime) -> Self {
		Self { runtime }
	}
}

use crate::{Handler, Runtime};

/// Interrupt resolution.
pub enum Resolve<'a, 'config, H: Handler> {
	/// Create interrupt resolution.
	Create(H::CreateInterrupt, ResolveCreate<'a, 'config>),
	/// Call interrupt resolution.
	Call(H::CallInterrupt, ResolveCall<'a, 'config>),
}

/// Create interrupt resolution.
#[allow(dead_code)]
pub struct ResolveCreate<'a, 'config> {
	runtime: &'a mut Runtime<'config>,
}

impl<'a, 'config> ResolveCreate<'a, 'config> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config>) -> Self {
		Self { runtime }
	}
}

/// Call interrupt resolution.
#[allow(dead_code)]
pub struct ResolveCall<'a, 'config> {
	runtime: &'a mut Runtime<'config>,
}

impl<'a, 'config> ResolveCall<'a, 'config> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config>) -> Self {
		Self { runtime }
	}
}

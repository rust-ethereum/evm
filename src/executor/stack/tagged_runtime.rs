//! A module containing data types for keeping track of the kinds of calls
//! (CALL vs CREATE) in the EVM call stack.

use crate::maybe_borrowed::MaybeBorrowed;
use crate::Runtime;
use primitive_types::H160;

pub struct TaggedRuntime<'config, 'borrow> {
	pub kind: RuntimeKind,
	pub inner: MaybeBorrowed<'borrow, Runtime<'config>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
	Create(H160),
	Call(H160),
	/// Special variant used only in `StackExecutor::execute`
	Execute,
}

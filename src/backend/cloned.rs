use crate::{TransactioalBackend, TransactionalMergeStrategy};

pub struct ClonedTransactionalBackend<H>(Vec<H>);

impl<H> TransactionalBackend for ClonedTransactionalBackend<H> {
	fn push_substate(&mut self) {

	}

	fn pop_substate(&mut self, strategy: TransactionalMergeStrategy) {

	}
}

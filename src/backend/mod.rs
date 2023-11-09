pub mod in_memory;

pub enum TransactionalMergeStrategy {
	Commit,
	Discard,
}

pub trait TransactionalBackend {
	fn push_substate(&mut self);
	fn pop_substate(&mut self, strategy: TransactionalMergeStrategy);
}

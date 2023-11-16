pub mod in_memory;

pub trait TransactionalBackend {
	fn push_substate(&mut self);
	fn pop_substate(&mut self, strategy: crate::MergeStrategy);
}

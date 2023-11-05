use crate::{MergeStrategy, RuntimeFullBackend};

pub trait TransactionalBackend: RuntimeFullBackend {
	fn push_substate(&mut self);
	fn pop_substate(&mut self, strategy: MergeStrategy);
}

use evm::interpreter::{etable::Control, machine::Machine};

pub fn eval_add<S, H, Tr>(
	_machine: &mut Machine<S>,
	_handle: &mut H,
	_position: usize,
) -> Control<Tr> {
	todo!()
}

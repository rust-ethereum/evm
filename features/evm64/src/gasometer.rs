use evm::{
	interpreter::{etable::Control, machine::Machine},
	standard::GasometerState,
};

pub fn eval<'config, S, H, Tr>(
	_machine: &mut Machine<S>,
	_handler: &mut H,
	_position: usize,
) -> Control<Tr>
where
	S: AsRef<GasometerState<'config>> + AsMut<GasometerState<'config>>,
{
	todo!()
}

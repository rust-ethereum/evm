use evm::interpreter::{
	Capture, ExitError, ExitResult, ExitSucceed, Interpreter,
	runtime::{
		Log, RuntimeBackend, RuntimeBaseBackend, RuntimeEnvironment, SetCodeOrigin, TouchKind,
	},
};
use evm::uint::{H160, H256, U256};
use evm_future::{FutureInterpreter, FutureInterpreterAction, FutureInterpreterSubmit};
use std::rc::Rc;

struct EmptyAction;

impl<S, H> FutureInterpreterAction<S, H> for EmptyAction {
	type Feedback = ();
	type Trap = ();

	fn run(self, _state: &mut S, _retbuf: &mut Vec<u8>, _handle: &mut H) -> Capture<(), ()> {
		Capture::Exit(())
	}
}

#[allow(clippy::unit_cmp)]
async fn async_precompile_noop(submit: Rc<FutureInterpreterSubmit<EmptyAction, ()>>) -> ExitResult {
	let feedback = submit.submit(EmptyAction).await;
	assert_eq!(feedback, ());
	Ok(ExitSucceed::Returned)
}

#[test]
fn create_future_closure() {
	let mut interpreter: FutureInterpreter<EmptyAction, (), _, ()> =
		FutureInterpreter::new((), Vec::new(), async_precompile_noop);
	let result = interpreter.run(&mut UnimplementedHandler);
	assert_eq!(result, Capture::Exit(ExitSucceed::Returned.into()));
}

pub struct UnimplementedHandler;

impl RuntimeEnvironment for UnimplementedHandler {
	fn block_hash(&self, _number: U256) -> H256 {
		unimplemented!()
	}
	fn block_number(&self) -> U256 {
		unimplemented!()
	}
	fn block_coinbase(&self) -> H160 {
		unimplemented!()
	}
	fn block_timestamp(&self) -> U256 {
		unimplemented!()
	}
	fn block_difficulty(&self) -> U256 {
		unimplemented!()
	}
	fn block_randomness(&self) -> Option<H256> {
		unimplemented!()
	}
	fn block_gas_limit(&self) -> U256 {
		unimplemented!()
	}
	fn block_base_fee_per_gas(&self) -> U256 {
		unimplemented!()
	}
	fn blob_base_fee_per_gas(&self) -> U256 {
		unimplemented!()
	}
	fn blob_versioned_hash(&self, _index: U256) -> H256 {
		unimplemented!()
	}
	fn chain_id(&self) -> U256 {
		unimplemented!()
	}
}

impl RuntimeBaseBackend for UnimplementedHandler {
	fn balance(&self, _address: H160) -> U256 {
		unimplemented!()
	}
	fn code_size(&self, _address: H160) -> U256 {
		unimplemented!()
	}
	fn code_hash(&self, _address: H160) -> H256 {
		unimplemented!()
	}
	fn code(&self, _address: H160) -> Vec<u8> {
		unimplemented!()
	}
	fn storage(&self, _address: H160, _index: H256) -> H256 {
		unimplemented!()
	}
	fn transient_storage(&self, _address: H160, _index: H256) -> H256 {
		unimplemented!()
	}

	fn exists(&self, _address: H160) -> bool {
		unimplemented!()
	}

	fn nonce(&self, _address: H160) -> U256 {
		unimplemented!()
	}
}

impl RuntimeBackend for UnimplementedHandler {
	fn original_storage(&self, _address: H160, _index: H256) -> H256 {
		unimplemented!()
	}

	fn deleted(&self, _address: H160) -> bool {
		unimplemented!()
	}

	fn created(&self, _address: H160) -> bool {
		unimplemented!()
	}

	fn is_cold(&self, _address: H160, _index: Option<H256>) -> bool {
		unimplemented!()
	}

	fn mark_hot(&mut self, _address: H160, _kind: TouchKind) {
		unimplemented!()
	}

	fn mark_storage_hot(&mut self, _address: H160, _index: H256) {
		unimplemented!()
	}

	fn set_storage(&mut self, _address: H160, _index: H256, _value: H256) -> Result<(), ExitError> {
		unimplemented!()
	}
	fn set_transient_storage(
		&mut self,
		_address: H160,
		_index: H256,
		_value: H256,
	) -> Result<(), ExitError> {
		unimplemented!()
	}
	fn log(&mut self, _log: Log) -> Result<(), ExitError> {
		unimplemented!()
	}
	fn mark_delete_reset(&mut self, _address: H160) {
		unimplemented!()
	}

	fn mark_create(&mut self, _address: H160) {
		unimplemented!()
	}

	fn reset_storage(&mut self, _address: H160) {
		unimplemented!()
	}

	fn set_code(
		&mut self,
		_address: H160,
		_code: Vec<u8>,
		_origin: SetCodeOrigin,
	) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn deposit(&mut self, _address: H160, _value: U256) {
		unimplemented!()
	}
	fn withdrawal(&mut self, _address: H160, _value: U256) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn inc_nonce(&mut self, _address: H160) -> Result<(), ExitError> {
		unimplemented!()
	}
}

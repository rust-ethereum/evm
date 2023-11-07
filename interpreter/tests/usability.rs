use evm_interpreter::{
	Capture, Context, Control, Etable, ExitError, ExitSucceed, Machine, Opcode, RuntimeBackend,
	RuntimeState,
};
use primitive_types::{H160, H256, U256};
use std::rc::Rc;

const CODE1: &str = "60e060020a6000350480632839e92814601e57806361047ff414603457005b602a6004356024356047565b8060005260206000f35b603d6004356099565b8060005260206000f35b600082600014605457605e565b8160010190506093565b81600014606957607b565b60756001840360016047565b90506093565b609060018403608c85600186036047565b6047565b90505b92915050565b6000816000148060a95750816001145b60b05760b7565b81905060cf565b60c1600283036099565b60cb600184036099565b0190505b91905056";
const DATA1: &str = "2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001";
const RET1: &str = "000000000000000000000000000000000000000000000000000000000000000d";

#[test]
fn etable_wrap() {
	let code = hex::decode(&CODE1).unwrap();
	let data = hex::decode(&DATA1).unwrap();

	let wrapped_etable = Etable::<_, _, Opcode>::core().wrap(|f, opcode_t| {
		move |machine, handle, opcode, position| {
			assert_eq!(opcode_t, opcode);
			println!("opcode: {:?}", opcode);
			f(machine, handle, opcode, position)
		}
	});

	let mut vm = Machine::new(Rc::new(code), Rc::new(data), 1024, 10000, ());
	let result = vm.run(&mut (), &wrapped_etable);
	assert_eq!(result, Capture::Exit(Ok(ExitSucceed::Returned.into())));
	assert_eq!(vm.into_retbuf(), hex::decode(&RET1).unwrap());
}

#[test]
fn etable_wrap2() {
	let code = hex::decode(&CODE1).unwrap();
	let data = hex::decode(&DATA1).unwrap();

	let wrapped_etable = Etable::core().wrap(
		|f, opcode_t| -> Box<dyn Fn(&mut Machine<()>, &mut (), Opcode, usize) -> Control<Opcode>> {
			if opcode_t != Opcode(0x50) {
				Box::new(move |machine, handle, opcode, position| {
					assert_eq!(opcode_t, opcode);
					println!("opcode: {:?}", opcode);
					f(machine, handle, opcode, position)
				})
			} else {
				Box::new(|_machine, _handle, opcode, _position| {
					println!("disabled!");
					Control::Trap(opcode)
				})
			}
		},
	);

	let mut vm = Machine::new(Rc::new(code), Rc::new(data), 1024, 10000, ());
	let result = vm.run(&mut (), &wrapped_etable);
	assert_eq!(result, Capture::Trap(Opcode(0x50)));
}

pub struct UnimplementedHandler;

impl<'a> RuntimeBackend for UnimplementedHandler {
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
	fn original_storage(&self, _address: H160, _index: H256) -> H256 {
		unimplemented!()
	}

	fn gas_price(&self) -> U256 {
		unimplemented!()
	}
	fn origin(&self) -> H160 {
		unimplemented!()
	}
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
	fn chain_id(&self) -> U256 {
		unimplemented!()
	}

	fn exists(&self, _address: H160) -> bool {
		unimplemented!()
	}
	fn deleted(&self, _address: H160) -> bool {
		unimplemented!()
	}
	fn is_cold(&self, _address: H160, _index: Option<H256>) -> bool {
		unimplemented!()
	}
	fn mark_hot(&mut self, _address: H160, _index: Option<H256>) -> Result<(), ExitError> {
		unimplemented!()
	}

	fn set_storage(&mut self, _address: H160, _index: H256, _value: H256) -> Result<(), ExitError> {
		unimplemented!()
	}
	fn log(&mut self, _address: H160, _topics: Vec<H256>, _data: Vec<u8>) -> Result<(), ExitError> {
		unimplemented!()
	}
	fn mark_delete(&mut self, _address: H160, _target: H160) -> Result<(), ExitError> {
		unimplemented!()
	}
}

struct DummyRuntimeState {
	context: Context,
	retbuf: Vec<u8>,
}

impl RuntimeState for DummyRuntimeState {
	fn context(&self) -> &Context {
		&self.context
	}

	fn retbuf(&self) -> &Vec<u8> {
		&self.retbuf
	}

	fn retbuf_mut(&mut self) -> &mut Vec<u8> {
		&mut self.retbuf
	}

	fn gas(&self) -> U256 {
		U256::zero()
	}
}

static RUNTIME_ETABLE: Etable<DummyRuntimeState, UnimplementedHandler, Opcode> = Etable::runtime();

#[test]
fn etable_runtime() {
	let code = hex::decode(&CODE1).unwrap();
	let data = hex::decode(&DATA1).unwrap();
	let mut handler = UnimplementedHandler;

	let mut vm = Machine::new(
		Rc::new(code),
		Rc::new(data),
		1024,
		10000,
		DummyRuntimeState {
			context: Context {
				address: H160::default(),
				caller: H160::default(),
				apparent_value: U256::default(),
			},
			retbuf: Vec::new(),
		},
	);

	let res = vm.run(&mut handler, &RUNTIME_ETABLE).exit().unwrap();
	assert_eq!(res, Ok(ExitSucceed::Returned.into()));
	assert_eq!(vm.into_retbuf(), hex::decode(&RET1).unwrap());
}

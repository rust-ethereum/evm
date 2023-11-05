use evm_interpreter::{Capture, Control, Etable, ExitSucceed, Machine, Opcode};
use std::rc::Rc;

const CODE1: &str = "60e060020a6000350480632839e92814601e57806361047ff414603457005b602a6004356024356047565b8060005260206000f35b603d6004356099565b8060005260206000f35b600082600014605457605e565b8160010190506093565b81600014606957607b565b60756001840360016047565b90506093565b609060018403608c85600186036047565b6047565b90505b92915050565b6000816000148060a95750816001145b60b05760b7565b81905060cf565b60c1600283036099565b60cb600184036099565b0190505b91905056";
const DATA1: &str = "2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001";
const RET1: &str = "000000000000000000000000000000000000000000000000000000000000000d";

#[test]
fn etable_wrap() {
	let code = hex::decode(&CODE1).unwrap();
	let data = hex::decode(&DATA1).unwrap();

	let wrapped_etable = Etable::core().wrap(|f, opcode_t| {
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
		|f, opcode_t| -> Box<dyn Fn(&mut Machine<()>, &mut (), Opcode, usize) -> Control> {
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

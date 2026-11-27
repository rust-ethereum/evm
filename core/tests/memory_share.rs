use evm_core::{CallInput, Capture, ExitSucceed, Machine, Memory};
use primitive_types::U256;
use std::rc::Rc;

#[test]
fn share_of_a_written_range_returns_its_bytes() {
	let mut memory = Memory::new(100);
	let bytes: Vec<u8> = (0..32).collect();
	memory.set(0, &bytes, None).unwrap();

	let input = memory.share(0, 32);
	assert_eq!(input.len(), 32);
	assert_eq!(&input[..], &bytes[..]);
	assert_eq!(input.to_vec(), bytes);

	let input = memory.share(8, 8);
	assert_eq!(input.len(), 8);
	assert_eq!(&input[..], &bytes[8..16]);
}

#[test]
fn share_past_the_physical_end_is_zero_padded() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3], None).unwrap();
	memory.resize_end(U256::from(64)).unwrap();

	// Partly past the physical end.
	let input = memory.share(0, 8);
	assert_eq!(&input[..], &memory.get(0, 8)[..]);
	assert_eq!(&input[..], &[1u8, 2, 3, 0, 0, 0, 0, 0]);

	// Wholly past the physical end.
	let input = memory.share(8, 8);
	assert_eq!(&input[..], &memory.get(8, 8)[..]);
	assert_eq!(&input[..], &[0u8; 8]);
}

#[test]
fn empty_share_touches_nothing() {
	let mut memory = Memory::new(100);
	assert_eq!(memory.len(), 0);

	let input = memory.share(0, 0);
	assert!(input.is_empty());
	assert_eq!(memory.len(), 0);
	assert!(memory.data().is_empty());

	memory.set(0, &[1u8, 2, 3, 4], None).unwrap();
	let data = memory.data().clone();

	let input = memory.share(2, 0);
	assert!(input.is_empty());
	assert_eq!(memory.len(), data.len());
	assert_eq!(memory.data(), &data);
}

#[test]
fn overflowing_range_falls_back_to_a_copy() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3], None).unwrap();

	let offset = usize::MAX - 1;
	let len = 4;
	let input = memory.share(offset, len);

	assert_eq!(&input[..], &memory.get(offset, len)[..]);
	assert_eq!(&input[..], &[0u8; 4]);

	// The copy does not grow the memory.
	assert_eq!(memory.data(), &[1u8, 2, 3].to_vec());
}

#[test]
fn set_after_the_share_ended_writes_in_place() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3, 4], None).unwrap();

	let input = memory.share(0, 4);
	assert_eq!(&input[..], &[1u8, 2, 3, 4]);
	drop(input);

	// The buffer is uniquely owned again, so the write does not copy it.
	let data = memory.data().as_ptr();
	memory.set(0, &[9u8, 9], None).unwrap();
	assert_eq!(memory.data().as_ptr(), data);
	assert_eq!(memory.data(), &[9u8, 9, 3, 4].to_vec());
}

#[test]
fn set_while_shared_keeps_the_shared_bytes() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3, 4], None).unwrap();

	let input = memory.share(0, 4);
	memory.set(0, &[9u8, 9], None).unwrap();

	assert_eq!(memory.data(), &[9u8, 9, 3, 4].to_vec());
	assert_eq!(&input[..], &[1u8, 2, 3, 4]);
}

#[test]
fn copy_after_the_share_ended_writes_in_place() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3, 4], None).unwrap();

	let input = memory.share(0, 4);
	drop(input);

	// The buffer is uniquely owned again, so the copy does not copy it.
	let data = memory.data().as_ptr();
	memory.copy(0, 2, 2);
	assert_eq!(memory.data().as_ptr(), data);
	assert_eq!(memory.data(), &[3u8, 4, 3, 4].to_vec());
}

#[test]
fn copy_while_shared_keeps_the_shared_bytes() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3, 4], None).unwrap();

	let input = memory.share(0, 4);
	memory.copy(0, 2, 2);

	assert_eq!(memory.data(), &[3u8, 4, 3, 4].to_vec());
	assert_eq!(&input[..], &[1u8, 2, 3, 4]);
}

#[test]
fn copy_overlapping_itself_matches_copy_within() {
	let mut memory = Memory::new(100);
	let bytes: Vec<u8> = (0..8).collect();
	memory.set(0, &bytes, None).unwrap();

	let mut expected = bytes.clone();
	expected.copy_within(0..6, 2);
	memory.copy(2, 0, 6);
	assert_eq!(memory.data(), &expected);

	expected.copy_within(2..8, 0);
	memory.copy(0, 2, 6);
	assert_eq!(memory.data(), &expected);
}

#[test]
fn second_share_inside_the_buffer_does_not_copy() {
	let mut memory = Memory::new(100);
	let bytes: Vec<u8> = (0..32).collect();
	memory.set(0, &bytes, None).unwrap();

	let first = memory.share(0, 16);
	let data = memory.data().as_ptr();
	let second = memory.share(8, 8);

	assert_eq!(memory.data().as_ptr(), data);
	assert_eq!(&first[..], &bytes[0..16]);
	assert_eq!(&second[..], &bytes[8..16]);
	assert_eq!(&first[8..16], &second[..]);
}

#[test]
fn default_input_is_empty() {
	let input = CallInput::default();

	assert!(input.is_empty());
	assert_eq!(input.len(), 0);
	assert_eq!(&input[..], &[] as &[u8]);
	assert!(input.to_vec().is_empty());
	assert_eq!(format!("{:?}", input), "[]");
}

#[test]
fn debug_of_memory_keeps_its_form() {
	assert_eq!(
		format!("{:?}", Memory::new(100)),
		format!(
			"Memory {{ data: [], effective_len: {:?}, limit: 100 }}",
			U256::zero()
		)
	);

	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3], None).unwrap();
	assert_eq!(
		format!("{:?}", memory),
		format!(
			"Memory {{ data: [1, 2, 3], effective_len: {:?}, limit: 100 }}",
			U256::zero()
		)
	);
}

#[test]
fn debug_of_call_input_matches_the_byte_slice() {
	let mut memory = Memory::new(100);
	memory.set(0, &[1u8, 2, 3], None).unwrap();

	let input = memory.share(0, 3);
	assert_eq!(format!("{:?}", input), format!("{:?}", vec![1u8, 2, 3]));
}

#[test]
fn memory_and_call_input_are_send_and_sync() {
	fn assert_send_sync<T: Send + Sync>() {}

	assert_send_sync::<Memory>();
	assert_send_sync::<CallInput>();
}

#[test]
fn machine_reads_call_data_from_a_shared_input() {
	// PUSH1 0; CALLDATALOAD; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; RETURN
	let code = hex::decode("60003560005260206000f3").unwrap();

	let mut memory = Memory::new(100);
	memory.set(0, &[0xaau8; 32], None).unwrap();
	let input = memory.share(0, 32);

	let mut machine = Machine::new_with_input(Rc::new(code), input, 1024, 10000);
	assert_eq!(machine.run(), Capture::Exit(ExitSucceed::Returned.into()));
	assert_eq!(machine.return_value(), vec![0xaau8; 32]);
}

#[test]
fn machine_with_an_empty_input_reports_its_size() {
	// CALLDATASIZE; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; RETURN
	let code = hex::decode("3660005260206000f3").unwrap();

	let mut machine = Machine::new_with_input(Rc::new(code), CallInput::default(), 1024, 10000);
	assert_eq!(machine.run(), Capture::Exit(ExitSucceed::Returned.into()));

	let ret = machine.return_value();
	assert_eq!(ret.len(), 32);
	assert_eq!(ret[31], 0);
}

use evm_core::{Capture, ExitSucceed, Machine};
use honggfuzz::fuzz;
use std::rc::Rc;

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
	return haystack
		.windows(needle.len())
		.position(|window| window == needle);
}

fn split_at_delim(sequence: &[u8], delim: &[u8]) -> (Vec<u8>, Vec<u8>) {
	let mut res_vec = Vec::new();
	let mut current_pos = 0;
	if let Some(index) = find_subsequence(&sequence[current_pos..], delim) {
		let found_index = index + current_pos;
		res_vec.push(sequence[current_pos..found_index].to_vec());
		current_pos = found_index + delim.len();
	}
	if current_pos == 0 {
		return (sequence.to_vec(), Vec::new());
	} else {
		res_vec.push(sequence[current_pos..].to_vec());
		return (res_vec[0].to_owned(), res_vec[1].to_owned());
	}
}

fn handle_data(sequence: &[u8]) {
	let (code, data) = split_at_delim(sequence, vec![0xde, 0xad, 0xbe, 0xef].as_slice());
	let stack_limit = 1024;
	let memory_limit = 10000;
	let mut vm = Machine::new(Rc::new(code), Rc::new(data), stack_limit, memory_limit);
	let res = vm.run();
	#[cfg(not(fuzzing))]
	{
		println!("Result: {:?}", res);
	}
}

fn main() {
	#[cfg(fuzzing)]
	{
		loop {
			fuzz!(|data: &[u8]| {
				handle_data(data);
			});
		}
	}
	#[cfg(not(fuzzing))]
	{
		use std::env;
		use std::fs;
		use std::fs::File;
		use std::io::Read;
		let args: Vec<_> = env::args().collect();
		let md = fs::metadata(&args[1]).unwrap();
		let all_files = match md.is_dir() {
			true => fs::read_dir(&args[1])
				.unwrap()
				.map(|x| x.unwrap().path().to_str().unwrap().to_string())
				.collect::<Vec<String>>(),
			false => (&args[1..]).to_vec(),
		};
		for argument in all_files {
			println!("Now doing file {:?}", argument);
			let mut buffer: Vec<u8> = Vec::new();
			let mut f = File::open(argument).unwrap();
			f.read_to_end(&mut buffer).unwrap();
			handle_data(buffer.as_slice());
		}
	}
}

// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! benchmarking for rlp
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

#![feature(test)]

extern crate test;
extern crate sputnikvm_rlp as rlp;

use test::Bencher;
use rlp::{RlpStream, Rlp};

#[bench]
fn bench_stream_u64_value(b: &mut Bencher) {
	b.iter(|| {
		// u64
		let mut stream = RlpStream::new();
		stream.append(&0x1023456789abcdefu64);
		let _ = stream.out();
	});
}

#[bench]
fn bench_decode_u64_value(b: &mut Bencher) {
	b.iter(|| {
		// u64
		let data = vec![0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
		let rlp = Rlp::new(&data);
		let _: u64 = rlp.as_val();
	});
}

#[bench]
fn bench_stream_nested_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		// [ [], [[]], [ [], [[]] ] ]
		let mut stream = RlpStream::new_list(3);
		stream.begin_list(0);
		stream.begin_list(1).begin_list(0);
		stream.begin_list(2).begin_list(0).begin_list(1).begin_list(0);
		let _ = stream.out();
	});
}

#[bench]
fn bench_decode_nested_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		// [ [], [[]], [ [], [[]] ] ]
		let data = vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0];
		let rlp = Rlp::new(&data);
		let _v0: Vec<u16> = rlp.at(0).as_list();
		let _v1: Vec<u16> = rlp.at(1).at(0).as_list();
		let nested_rlp = rlp.at(2);
		let _v2a: Vec<u16> = nested_rlp.at(0).as_list();
		let _v2b: Vec<u16> = nested_rlp.at(1).at(0).as_list();
	});
}

#[bench]
fn bench_stream_1000_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		let mut stream = RlpStream::new_list(1000);
		for _ in 0..1000 {
			stream.begin_list(0);
		}
		let _ = stream.out();
	});
}

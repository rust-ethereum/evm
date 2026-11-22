//! The tracing event of a call carries the input the callee reads.

#![cfg(feature = "tracing")]

mod common;

use common::*;
use evm::tracing::{using, Event, EventListener};
use evm::{Config, ExitReason, ExitSucceed};

/// A listener that records the input of every call.
#[derive(Default)]
struct Recorder {
	inputs: Vec<Vec<u8>>,
}

impl EventListener for Recorder {
	fn event(&mut self, event: Event<'_>) {
		if let Event::Call { input, .. } = event {
			self.inputs.push(input.to_vec());
		}
	}
}

#[test]
fn the_call_event_carries_the_input_the_callee_reads() {
	let payload = payload(64);
	// The contract passes the range [16, 48) of its memory, which is the
	// middle of the calldata it copied there.
	let code = caller(0xf1, ECHO_ADDR, 16, 32, 0x1000, 32);
	let state = state(&[(CONTRACT, code), (ECHO_ADDR, decode_hex(ECHO))]);

	let mut recorder = Recorder::default();
	let (reason, output) = using(&mut recorder, || {
		run(state, payload.clone(), &Config::cancun())
	});

	assert_eq!(reason, ExitReason::Succeed(ExitSucceed::Returned));

	// The transaction's own call comes first, then the contract's call.
	let inputs = &recorder.inputs;
	assert_eq!(inputs.len(), 2, "the number of call events");
	assert_eq!(inputs[0], payload, "the transaction's call input");
	assert_eq!(
		inputs[1],
		payload[16..48].to_vec(),
		"the contract's call input"
	);

	// The callee echoed exactly the bytes the event reported.
	assert_eq!(&output[..32], &word(1)[..], "the call's success flag");
	assert_eq!(&output[32..], &inputs[1][..], "the bytes the callee read");
}

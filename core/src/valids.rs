use crate::Opcode;
use elrond_wasm::{api::ManagedTypeApi, types::ManagedVec};

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Valids<M: ManagedTypeApi>(ManagedVec<M, bool>);

impl<M: ManagedTypeApi> Valids<M> {
	/// Create a new valid mapping from given code bytes.
	pub fn new(code: &ManagedVec<M, u8>) -> Self {
		let mut valids: ManagedVec<M, bool> = ManagedVec::new();
		// valids.resize(code.len(), false);

		let mut i = 0;
		while i < code.len() {
			let opcode = Opcode(code.get(i));
			if opcode == Opcode::JUMPDEST {
				valids.set(i, &true);
				i += 1;
			} else if let Some(v) = opcode.is_push() {
				i += v as usize + 1;
			} else {
				i += 1;
			}
		}

		Valids(valids)
	}

	/// Get the length of the valid mapping. This is the same as the
	/// code bytes.
	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Returns true if the valids list is empty
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns `true` if the position is a valid jump destination. If
	/// not, returns `false`.
	pub fn is_valid(&self, position: usize) -> bool {
		if position >= self.0.len() {
			return false;
		}

		if !self.0.get(position) {
			return false;
		}

		true
	}
}

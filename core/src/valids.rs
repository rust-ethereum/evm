use alloc::vec::Vec;
use crate::Opcode;

/// A valid marker.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Valid {
	/// Invalid position.
	None,
	/// Valid jump destination.
	JumpDest,
	/// Valid begin subroutine.
	BeginSub,
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Valids(Vec<Valid>);

impl Valids {
	/// Create a new valid mapping from given code bytes.
	pub fn new(code: &[u8]) -> Self {
		let mut valids: Vec<Valid> = Vec::with_capacity(code.len());
		valids.resize(code.len(), Valid::None);

		let mut i = 0;
		while i < code.len() {
			let opcode = Opcode(code[i]);
			if opcode == Opcode::JUMPDEST {
				valids[i] = Valid::JumpDest;
				i += 1;
			} else if opcode == Opcode::BEGINSUB {
				valids[i] = Valid::BeginSub;
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
	pub fn is_jumpdest(&self, position: usize) -> bool {
		if position >= self.0.len() {
			return false
		}

		self.0[position] == Valid::JumpDest
	}

	/// Returns `true` if the position is a valid begin subroutine. If
	/// not, returns `false`.
	pub fn is_beginsub(&self, position: usize) -> bool {
		if position >= self.0.len() {
			return false
		}

		self.0[position] == Valid::BeginSub
	}
}

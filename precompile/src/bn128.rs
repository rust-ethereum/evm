use alloc::vec::Vec;

use evm::{
	interpreter::error::{ExitError, ExitException, ExitResult, ExitSucceed},
	GasMutState,
};
use primitive_types::U256;

use crate::PurePrecompile;

/// Copy bytes from input to target.
fn read_input(source: &[u8], target: &mut [u8], offset: usize) {
	// Out of bounds, nothing to copy.
	if source.len() <= offset {
		return;
	}

	// Find len to copy up to target len, but not out of bounds.
	let len = core::cmp::min(target.len(), source.len() - offset);
	target[..len].copy_from_slice(&source[offset..][..len]);
}

fn read_fr(input: &[u8], start_inx: usize) -> Result<bn::Fr, ExitError> {
	let mut buf = [0u8; 32];
	read_input(input, &mut buf, start_inx);

	let ret = bn::Fr::from_slice(&buf)
		.map_err(|_| ExitException::Other("Invalid field element".into()))?;
	Ok(ret)
}

fn read_point(input: &[u8], start_inx: usize) -> Result<bn::G1, ExitError> {
	use bn::{AffineG1, Fq, Group, G1};

	let mut px_buf = [0u8; 32];
	let mut py_buf = [0u8; 32];
	read_input(input, &mut px_buf, start_inx);
	read_input(input, &mut py_buf, start_inx + 32);

	let px = Fq::from_slice(&px_buf)
		.map_err(|_| ExitException::Other("Invalid point x coordinate".into()))?;

	let py = Fq::from_slice(&py_buf)
		.map_err(|_| ExitException::Other("Invalid point y coordinate".into()))?;

	Ok(if px == Fq::zero() && py == Fq::zero() {
		G1::zero()
	} else {
		AffineG1::new(px, py)
			.map_err(|_| ExitException::Other("Invalid curve point".into()))?
			.into()
	})
}

/// The Bn128Add builtin
pub struct Bn128Add;

impl Bn128Add {
	const GAS_COST: u64 = 150; // https://eips.ethereum.org/EIPS/eip-1108
}

impl<G: GasMutState> PurePrecompile<G> for Bn128Add {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		use bn::AffineG1;

		try_some!(gasometer.record_gas(Bn128Add::GAS_COST.into()));

		let p1 = try_some!(read_point(input, 0));
		let p2 = try_some!(read_point(input, 64));

		let mut buf = [0u8; 64];
		if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
			// point not at infinity
			try_some!(sum
				.x()
				.to_big_endian(&mut buf[0..32])
				.map_err(|_| ExitException::Other(
					"Cannot fail since 0..32 is 32-byte length".into()
				)));
			try_some!(sum.y().to_big_endian(&mut buf[32..64]).map_err(|_| {
				ExitException::Other("Cannot fail since 32..64 is 32-byte length".into())
			}));
		}

		(ExitSucceed::Returned.into(), buf.to_vec())
	}
}

/// The Bn128Mul builtin
pub struct Bn128Mul;

impl Bn128Mul {
	const GAS_COST: u64 = 6_000; // https://eips.ethereum.org/EIPS/eip-1108
}

impl<G: GasMutState> PurePrecompile<G> for Bn128Mul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		use bn::AffineG1;

		try_some!(gasometer.record_gas(Bn128Mul::GAS_COST.into()));

		let p = try_some!(read_point(input, 0));
		let fr = try_some!(read_fr(input, 64));

		let mut buf = [0u8; 64];
		if let Some(sum) = AffineG1::from_jacobian(p * fr) {
			// point not at infinity
			try_some!(sum
				.x()
				.to_big_endian(&mut buf[0..32])
				.map_err(|_| ExitException::Other(
					"Cannot fail since 0..32 is 32-byte length".into()
				)));
			try_some!(sum.y().to_big_endian(&mut buf[32..64]).map_err(|_| {
				ExitException::Other("Cannot fail since 32..64 is 32-byte length".into())
			}));
		}

		(ExitSucceed::Returned.into(), buf.to_vec())
	}
}

/// The Bn128Pairing builtin
pub struct Bn128Pairing;

impl Bn128Pairing {
	// https://eips.ethereum.org/EIPS/eip-1108
	const BASE_GAS_COST: u64 = 45_000;
	const GAS_COST_PER_PAIRING: u64 = 34_000;
}

impl<G: GasMutState> PurePrecompile<G> for Bn128Pairing {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		use bn::{pairing_batch, AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

		let ret_val = if input.is_empty() {
			try_some!(gasometer.record_gas(Bn128Pairing::BASE_GAS_COST.into()));
			U256::one()
		} else {
			if input.len() % 192 > 0 {
				return (
					ExitException::Other("bad elliptic curve pairing size".into()).into(),
					Vec::new(),
				);
			}

			// (a, b_a, b_b - each 64-byte affine coordinates)
			let elements = input.len() / 192;

			let gas_cost: u64 = Bn128Pairing::BASE_GAS_COST
				+ (elements as u64 * Bn128Pairing::GAS_COST_PER_PAIRING);

			try_some!(gasometer.record_gas(gas_cost.into()));

			let mut vals = Vec::new();
			for idx in 0..elements {
				let a_x = try_some!(Fq::from_slice(&input[idx * 192..idx * 192 + 32])
					.map_err(|_| ExitException::Other("Invalid a argument x coordinate".into())));

				let a_y = try_some!(Fq::from_slice(&input[idx * 192 + 32..idx * 192 + 64])
					.map_err(|_| ExitException::Other("Invalid a argument y coordinate".into(),)));

				let b_a_y = try_some!(Fq::from_slice(&input[idx * 192 + 64..idx * 192 + 96])
					.map_err(|_| {
						ExitException::Other(
							"Invalid b argument imaginary coeff x coordinate".into(),
						)
					}));

				let b_a_x = try_some!(Fq::from_slice(&input[idx * 192 + 96..idx * 192 + 128])
					.map_err(|_| ExitException::Other(
						"Invalid b argument imaginary coeff y coordinate".into(),
					)));

				let b_b_y = try_some!(Fq::from_slice(&input[idx * 192 + 128..idx * 192 + 160])
					.map_err(|_| {
						ExitException::Other("Invalid b argument real coeff x coordinate".into())
					}));

				let b_b_x = try_some!(Fq::from_slice(&input[idx * 192 + 160..idx * 192 + 192])
					.map_err(|_| {
						ExitException::Other("Invalid b argument real coeff y coordinate".into())
					}));

				let b_a = Fq2::new(b_a_x, b_a_y);
				let b_b = Fq2::new(b_b_x, b_b_y);
				let b = if b_a.is_zero() && b_b.is_zero() {
					G2::zero()
				} else {
					G2::from(try_some!(AffineG2::new(b_a, b_b).map_err(|_| {
						ExitException::Other("Invalid b argument - not on curve".into())
					})))
				};
				let a = if a_x.is_zero() && a_y.is_zero() {
					G1::zero()
				} else {
					G1::from(try_some!(AffineG1::new(a_x, a_y).map_err(|_| {
						ExitException::Other("Invalid a argument - not on curve".into())
					},)))
				};
				vals.push((a, b));
			}

			let mul = pairing_batch(&vals);

			if mul == Gt::one() {
				U256::one()
			} else {
				U256::zero()
			}
		};

		let mut buf = [0u8; 32];
		ret_val.to_big_endian(&mut buf);

		(ExitSucceed::Returned.into(), buf.to_vec())
	}
}

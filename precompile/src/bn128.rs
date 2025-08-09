use alloc::{vec::Vec, borrow::Cow};
use evm::{
	interpreter::{error::{ExitError, ExitException, ExitResult, ExitSucceed}, utils::u256_to_h256},
	GasMutState,
};
use primitive_types::U256;
use bn::{AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

use crate::PurePrecompile;

pub struct Bn128AddIstanbul;

impl<G: GasMutState> PurePrecompile<G> for Bn128AddIstanbul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const ISTANBUL_ADD_GAS_COST: u64 = 150;

		match run_add(input, ISTANBUL_ADD_GAS_COST, gasometer) {
			Ok((res, out)) => (Ok(res), out),
			Err(err) => (Err(err), Vec::new()),
		}
	}
}

pub struct Bn128MulIstanbul;

impl<G: GasMutState> PurePrecompile<G> for Bn128MulIstanbul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		const ISTANBUL_MUL_GAS_COST: u64 = 6_000;

		match run_mul(input, ISTANBUL_MUL_GAS_COST, gasometer) {
			Ok((res, out)) => (Ok(res), out),
			Err(err) => (Err(err), Vec::new()),
		}
	}
}

pub struct Bn128PairingIstanbul;

impl<G: GasMutState> PurePrecompile<G> for Bn128PairingIstanbul {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		/// Bn254 pair precompile with ISTANBUL gas rules
		const ISTANBUL_PAIR_PER_POINT: u64 = 34_000;

		/// Bn254 pair precompile with ISTANBUL gas rules
		const ISTANBUL_PAIR_BASE: u64 = 45_000;

		match run_pair(input, ISTANBUL_PAIR_PER_POINT, ISTANBUL_PAIR_BASE, gasometer) {
			Ok((res, out)) => (Ok(res), out),
			Err(err) => (Err(err), Vec::new()),
		}
	}
}

/// FQ_LEN specifies the number of bytes needed to represent an
/// Fq element. This is an element in the base field of BN254.
///
/// Note: The base field is used to define G1 and G2 elements.
const FQ_LEN: usize = 32;

/// SCALAR_LEN specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BN254.
const SCALAR_LEN: usize = 32;

/// FQ2_LEN specifies the number of bytes needed to represent an
/// Fq^2 element.
///
/// Note: This is the quadratic extension of Fq, and by definition
/// means we need 2 Fq elements.
const FQ2_LEN: usize = 2 * FQ_LEN;

/// G1_LEN specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fq elements.
const G1_LEN: usize = 2 * FQ_LEN;
/// G2_LEN specifies the number of bytes needed to represent a G2 element.
///
/// Note: A G2 element contains 2 Fq^2 elements.
const G2_LEN: usize = 2 * FQ2_LEN;

/// Input length for the add operation.
/// `ADD` takes two uncompressed G1 points (64 bytes each).
pub const ADD_INPUT_LEN: usize = 2 * G1_LEN;

/// Input length for the multiplication operation.
/// `MUL` takes an uncompressed G1 point (64 bytes) and scalar (32 bytes).
pub const MUL_INPUT_LEN: usize = G1_LEN + SCALAR_LEN;

/// Pair element length.
/// `PAIR` elements are composed of an uncompressed G1 point (64 bytes) and an uncompressed G2 point
/// (128 bytes).
pub const PAIR_ELEMENT_LEN: usize = G1_LEN + G2_LEN;

/// Run the Bn254 add precompile
pub fn run_add<G: GasMutState>(input: &[u8], gas_cost: u64, gasometer: &mut G) -> Result<(ExitSucceed, Vec<u8>), ExitError> {
	gasometer.record_gas(gas_cost.into())?;

    let input = right_pad::<ADD_INPUT_LEN>(input);

    let p1_bytes = &input[..G1_LEN];
    let p2_bytes = &input[G1_LEN..];
    let output = g1_point_add(p1_bytes, p2_bytes)?;

	Ok((ExitSucceed::Returned.into(), output.into()))
}

/// Run the Bn254 mul precompile
pub fn run_mul<G: GasMutState>(input: &[u8], gas_cost: u64, gasometer: &mut G) -> Result<(ExitSucceed, Vec<u8>), ExitError> {
	gasometer.record_gas(gas_cost.into())?;

    let input = right_pad::<MUL_INPUT_LEN>(input);

    let point_bytes = &input[..G1_LEN];
    let scalar_bytes = &input[G1_LEN..G1_LEN + SCALAR_LEN];
    let output = g1_point_mul(point_bytes, scalar_bytes)?;

	Ok((ExitSucceed::Returned.into(), output.into()))
}

/// Run the Bn254 pair precompile
pub fn run_pair<G: GasMutState>(
    input: &[u8],
    pair_per_point_cost: u64,
    pair_base_cost: u64,
	gasometer: &mut G,
) -> Result<(ExitSucceed, Vec<u8>), ExitError> {
    let gas_used = (input.len() / PAIR_ELEMENT_LEN) as u64 * pair_per_point_cost + pair_base_cost;
	gasometer.record_gas(gas_used.into())?;

    if !input.len().is_multiple_of(PAIR_ELEMENT_LEN) {
        return Err(ExitException::OutOfGas.into());
    }

    let elements = input.len() / PAIR_ELEMENT_LEN;

    let mut points = Vec::with_capacity(elements);

    for idx in 0..elements {
        // Offset to the start of the pairing element at index `idx` in the byte slice
        let start = idx * PAIR_ELEMENT_LEN;
        let g1_start = start;
        // Offset to the start of the G2 element in the pairing element
        // This is where G1 ends.
        let g2_start = start + G1_LEN;

        // Get G1 and G2 points from the input
        let encoded_g1_element = &input[g1_start..g2_start];
        let encoded_g2_element = &input[g2_start..g2_start + G2_LEN];
        points.push((encoded_g1_element, encoded_g2_element));
    }

    let pairing_result = pairing_check(&points)?;
    Ok((
		ExitSucceed::Returned.into(),
        u256_to_h256(bool_to_u256(pairing_result)).0.into(),
    ))
}

/// Reads a single `Fq` field element from the input slice.
///
/// Takes a byte slice and attempts to interpret the first 32 bytes as an
/// elliptic curve field element. Returns an error if the bytes do not form
/// a valid field element.
///
/// # Panics
///
/// Panics if the input is not at least 32 bytes long.
#[inline]
fn read_fq(input: &[u8]) -> Result<Fq, ExitError> {
	Ok(Fq::from_slice(&input[..FQ_LEN]).map_err(|_| ExitException::OutOfGas)?)
}
/// Reads a Fq2 (quadratic extension field element) from the input slice.
///
/// Parses two consecutive Fq field elements as the real and imaginary parts
/// of an Fq2 element.
/// The second component is parsed before the first, ie if a we represent an
/// element in Fq2 as (x,y) -- `y` is parsed before `x`
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
fn read_fq2(input: &[u8]) -> Result<Fq2, ExitError> {
    let y = read_fq(&input[..FQ_LEN])?;
    let x = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
    Ok(Fq2::new(x, y))
}

/// Creates a new `G1` point from the given `x` and `y` coordinates.
///
/// Constructs a point on the G1 curve from its affine coordinates.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically because `AffineG1` is not capable of
/// representing such a point.
/// In particular, when we convert from `AffineG1` to `G1`, the point
/// will be (0,0,1) instead of (0,1,0)
#[inline]
fn new_g1_point(px: Fq, py: Fq) -> Result<G1, ExitError> {
    if px == Fq::zero() && py == Fq::zero() {
        Ok(G1::zero())
    } else {
        Ok(AffineG1::new(px, py)
           .map(Into::into)
           .map_err(|_| ExitException::OutOfGas)?)
    }
}

/// Creates a new `G2` point from the given Fq2 coordinates.
///
/// G2 points in BN254 are defined over a quadratic extension field Fq2.
/// This function takes two Fq2 elements representing the x and y coordinates
/// and creates a G2 point.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically because `AffineG2` is not capable of
/// representing such a point.
/// In particular, when we convert from `AffineG2` to `G2`, the point
/// will be (0,0,1) instead of (0,1,0)
#[inline]
fn new_g2_point(x: Fq2, y: Fq2) -> Result<G2, ExitError> {
    let point = if x.is_zero() && y.is_zero() {
        G2::zero()
    } else {
        G2::from(AffineG2::new(x, y).map_err(|_| ExitException::OutOfGas)?)
    };

    Ok(point)
}

/// Reads a G1 point from the input slice.
///
/// Parses a G1 point from a byte slice by reading two consecutive field elements
/// representing the x and y coordinates.
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
pub fn read_g1_point(input: &[u8]) -> Result<G1, ExitError> {
    let px = read_fq(&input[0..FQ_LEN])?;
    let py = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
    new_g1_point(px, py)
}

/// Encodes a G1 point into a byte array.
///
/// Converts a G1 point in Jacobian coordinates to affine coordinates and
/// serializes the x and y coordinates as big-endian byte arrays.
///
/// Note: If the point is the point at infinity, this function returns
/// all zeroes.
#[inline]
pub fn encode_g1_point(point: G1) -> [u8; G1_LEN] {
    let mut output = [0u8; G1_LEN];

    if let Some(point_affine) = AffineG1::from_jacobian(point) {
        point_affine
            .x()
            .to_big_endian(&mut output[..FQ_LEN])
            .unwrap();
        point_affine
            .y()
            .to_big_endian(&mut output[FQ_LEN..])
            .unwrap();
    }

    output
}

/// Reads a G2 point from the input slice.
///
/// Parses a G2 point from a byte slice by reading four consecutive Fq field elements
/// representing the two Fq2 coordinates (x and y) of the G2 point.
///
/// # Panics
///
/// Panics if the input is not at least 128 bytes long.
#[inline]
pub fn read_g2_point(input: &[u8]) -> Result<G2, ExitError> {
    let ba = read_fq2(&input[0..FQ2_LEN])?;
    let bb = read_fq2(&input[FQ2_LEN..2 * FQ2_LEN])?;
    new_g2_point(ba, bb)
}

/// Reads a scalar from the input slice
///
/// Note: The scalar does not need to be canonical.
///
/// # Panics
///
/// If `input.len()` is not equal to [`SCALAR_LEN`].
#[inline]
pub fn read_scalar(input: &[u8]) -> bn::Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    // `Fr::from_slice` can only fail when the length is not `SCALAR_LEN`.
    bn::Fr::from_slice(input).unwrap()
}

/// Performs point addition on two G1 points.
#[inline]
fn g1_point_add(p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], ExitError> {
    let p1 = read_g1_point(p1_bytes)?;
    let p2 = read_g1_point(p2_bytes)?;
    let result = p1 + p2;
    Ok(encode_g1_point(result))
}

/// Performs a G1 scalar multiplication.
#[inline]
fn g1_point_mul(
    point_bytes: &[u8],
    fr_bytes: &[u8],
) -> Result<[u8; 64], ExitError> {
    let p = read_g1_point(point_bytes)?;
    let fr = read_scalar(fr_bytes);
    let result = p * fr;
    Ok(encode_g1_point(result))
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
///
/// Note: If the input is empty, this function returns true.
/// This is different to EIP2537 which disallows the empty input.
#[inline]
fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, ExitError> {
    let mut parsed_pairs = Vec::with_capacity(pairs.len());

    for (g1_bytes, g2_bytes) in pairs {
        let g1 = read_g1_point(g1_bytes)?;
        let g2 = read_g2_point(g2_bytes)?;

        // Skip pairs where either point is at infinity
        if !g1.is_zero() && !g2.is_zero() {
            parsed_pairs.push((g1, g2));
        }
    }

    if parsed_pairs.is_empty() {
        return Ok(true);
    }

    Ok(bn::pairing_batch(&parsed_pairs) == Gt::one())
}

/// Right-pads the given slice with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline]
fn right_pad<const LEN: usize>(data: &[u8]) -> Cow<'_, [u8; LEN]> {
    if let Some(data) = data.get(..LEN) {
        Cow::Borrowed(data.try_into().unwrap())
    } else {
        let mut padded = [0; LEN];
        padded[..data.len()].copy_from_slice(data);
        Cow::Owned(padded)
    }
}

#[inline]
pub const fn bool_to_u256(value: bool) -> U256 {
    if value {
		U256::one()
    } else {
        U256::zero()
    }
}

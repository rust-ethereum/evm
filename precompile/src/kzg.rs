//! KZG point evaluation precompile using Arkworks BLS12-381 implementation.
use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup, pairing::Pairing};
use ark_ff::{BigInteger, One, PrimeField};
use ark_serialize::CanonicalDeserialize;
use core::ops::Neg;
use evm::{
	GasMutState,
	interpreter::{ExitError, ExitException, ExitResult, ExitSucceed},
};
use primitive_types::U256;
use sha2::Digest;

use crate::PurePrecompile;

/// Gas cost of the KZG point evaluation precompile.
pub const GAS_COST: u64 = 50_000;

/// Versioned hash version for KZG.
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// `U256(FIELD_ELEMENTS_PER_BLOB).to_be_bytes() ++ BLS_MODULUS.to_bytes32()`
pub const RETURN_VALUE: &[u8; 64] = &hex_literal::hex!(
	"0000000000000000000000000000000000000000000000000000000000001000"
	"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
);

pub struct KZGPointEvaluation;

impl<G: GasMutState> PurePrecompile<G> for KZGPointEvaluation {
	fn execute(&self, input: &[u8], gasometer: &mut G) -> (ExitResult, Vec<u8>) {
		try_some!(gasometer.record_gas(U256::from(GAS_COST)));

		match run(input) {
			Ok(ret) => (ExitSucceed::Returned.into(), ret),
			Err(e) => (e.into(), Vec::new()),
		}
	}
}

/// Run kzg point evaluation precompile.
///
/// The Env has the KZGSettings that is needed for evaluation.
///
/// The input is encoded as follows:
/// | versioned_hash |  z  |  y  | commitment | proof |
/// |     32         | 32  | 32  |     48     |   48  |
/// with z and y being padded 32 byte big endian values
pub fn run(input: &[u8]) -> Result<Vec<u8>, ExitError> {
	// Verify input length.
	if input.len() != 192 {
		return Err(ExitException::Other("BlobInvalidInputLength".into()).into());
	}

	// Verify commitment matches versioned_hash
	let versioned_hash = &input[..32];
	let commitment = &input[96..144];
	if kzg_to_versioned_hash(commitment) != versioned_hash {
		return Err(ExitException::Other("BlobMismatchedVersion".into()).into());
	}

	// Verify KZG proof with z and y in big endian format
	let commitment: &[u8; 48] = commitment.try_into().unwrap();
	let z = input[32..64].try_into().unwrap();
	let y = input[64..96].try_into().unwrap();
	let proof = input[144..192].try_into().unwrap();
	if verify_kzg_proof(commitment, z, y, proof) {
		// Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values
		Ok(RETURN_VALUE.into())
	} else {
		Err(ExitException::Other("BlobVerifyKzgProofFailed".into()).into())
	}
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
#[inline]
pub fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
	let mut hash: [u8; 32] = sha2::Sha256::digest(commitment).into();
	hash[0] = VERSIONED_HASH_VERSION_KZG;
	hash
}

/// Verify KZG proof using BLS12-381 implementation.
///
/// <https://github.com/ethereum/EIPs/blob/4d2a00692bb131366ede1a16eced2b0e25b1bf99/EIPS/eip-4844.md?plain=1#L203>
/// <https://github.com/ethereum/consensus-specs/blob/master/specs/deneb/polynomial-commitments.md#verify_kzg_proof_impl>
#[inline]
pub fn verify_kzg_proof(
	commitment: &[u8; 48],
	z: &[u8; 32],
	y: &[u8; 32],
	proof: &[u8; 48],
) -> bool {
	// Parse the commitment (G1 point)
	let Ok(commitment_point) = parse_g1_compressed(commitment) else {
		return false;
	};

	// Parse the proof (G1 point)
	let Ok(proof_point) = parse_g1_compressed(proof) else {
		return false;
	};

	// Parse z and y as field elements (Fr, scalar field)
	// We expect 32-byte big-endian scalars that must be canonical
	let Ok(z_fr) = read_scalar_canonical(z) else {
		return false;
	};
	let Ok(y_fr) = read_scalar_canonical(y) else {
		return false;
	};

	// Get the trusted setup G2 point [τ]₂
	let tau_g2 = get_trusted_setup_g2();

	// Get generators
	let g1 = get_g1_generator();
	let g2 = get_g2_generator();

	// Compute P_minus_y = commitment - [y]G₁
	let y_g1 = p1_scalar_mul(&g1, &y_fr);
	let p_minus_y = p1_sub_affine(&commitment_point, &y_g1);

	// Compute X_minus_z = [τ]G₂ - [z]G₂
	let z_g2 = p2_scalar_mul(&g2, &z_fr);
	let x_minus_z = p2_sub_affine(&tau_g2, &z_g2);

	// Verify: P - y = Q * (X - z)
	// Using pairing check: e(P - y, -G₂) * e(proof, X - z) == 1
	let neg_g2 = p2_neg(&g2);

	pairing_check(&[(p_minus_y, neg_g2), (proof_point, x_minus_z)])
}

/// Get the trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> G2Affine {
	// Parse the compressed G2 point using unchecked deserialization since we trust this point
	// This should never fail since we're using a known valid point from the trusted setup
	G2Affine::deserialize_compressed_unchecked(&TRUSTED_SETUP_TAU_G2_BYTES[..])
		.expect("Failed to parse trusted setup G2 point")
}

/// Parse a G1 point from compressed format (48 bytes)
fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<G1Affine, ExitError> {
	let g1 = G1Affine::deserialize_compressed(&bytes[..])
		.map_err(|_| ExitException::Other("Invalid compressed G1 point".into()))?;
	Ok(g1)
}

/// Read a scalar field element from bytes and verify it's canonical
fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<Fr, ExitError> {
	let fr = Fr::from_be_bytes_mod_order(bytes);

	// Check if the field element is canonical by serializing back and comparing
	let bytes_roundtrip = fr.into_bigint().to_bytes_be();

	if bytes_roundtrip.as_slice() != bytes {
		return Err(ExitException::Other("Non-canonical scalar field element".into()).into());
	}

	Ok(fr)
}

/// Get G1 generator point
#[inline]
fn get_g1_generator() -> G1Affine {
	G1Affine::generator()
}

/// Get G2 generator point
#[inline]
fn get_g2_generator() -> G2Affine {
	G2Affine::generator()
}

/// Scalar multiplication for G1 points
#[inline]
fn p1_scalar_mul(point: &G1Affine, scalar: &Fr) -> G1Affine {
	point.mul_bigint(scalar.into_bigint()).into_affine()
}

/// Scalar multiplication for G2 points
#[inline]
fn p2_scalar_mul(point: &G2Affine, scalar: &Fr) -> G2Affine {
	point.mul_bigint(scalar.into_bigint()).into_affine()
}

/// Subtract two G1 points in affine form
#[inline]
fn p1_sub_affine(a: &G1Affine, b: &G1Affine) -> G1Affine {
	(a.into_group() - b.into_group()).into_affine()
}

/// Subtract two G2 points in affine form
#[inline]
fn p2_sub_affine(a: &G2Affine, b: &G2Affine) -> G2Affine {
	(a.into_group() - b.into_group()).into_affine()
}

/// Negate a G2 point
#[inline]
fn p2_neg(p: &G2Affine) -> G2Affine {
	p.neg()
}

/// The trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony (compressed format)
/// Taken from: <https://github.com/ethereum/consensus-specs/blob/adc514a1c29532ebc1a67c71dc8741a2fdac5ed4/presets/mainnet/trusted_setups/trusted_setup_4096.json#L8200C6-L8200C200>
const TRUSTED_SETUP_TAU_G2_BYTES: [u8; 96] = hex_literal::hex!(
	"b5bfd7dd8cdeb128843bc287230af38926187075cbfbefa81009a2ce615ac53d2914e5870cb452d2afaaab24f3499f72185cbfee53492714734429b7b38608e23926c911cceceac9a36851477ba4c60b087041de621000edc98edada20c1def2"
);

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
#[inline]
fn pairing_check(pairs: &[(G1Affine, G2Affine)]) -> bool {
	if pairs.is_empty() {
		return true;
	}

	let (g1_points, g2_points): (Vec<G1Affine>, Vec<G2Affine>) = pairs.iter().copied().unzip();

	let pairing_result = Bls12_381::multi_pairing(&g1_points, &g2_points);
	pairing_result.0.is_one()
}

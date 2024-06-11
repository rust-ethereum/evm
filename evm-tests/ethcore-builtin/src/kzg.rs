use c_kzg::{Bytes32, Bytes48, KzgProof, KzgSettings, BYTES_PER_G1_POINT, BYTES_PER_G2_POINT};
use core::convert::TryInto;
use core::hash::{Hash, Hasher};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use hex_literal::hex;
use sha2::Digest;
use std::convert::TryFrom;
use std::rc::Rc;

pub const RETURN_VALUE: &[u8; 64] = &hex!(
	"0000000000000000000000000000000000000000000000000000000000001000"
	"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
);

/// Number of G1 Points.
const NUM_G1_POINTS: usize = 4096;

/// Number of G2 Points.
const NUM_G2_POINTS: usize = 65;

/// A newtype over list of G1 point from kzg trusted setup.
#[derive(Debug, Clone, PartialEq, AsRef, AsMut, Deref, DerefMut)]
#[repr(transparent)]
struct G1Points(pub [[u8; BYTES_PER_G1_POINT]; NUM_G1_POINTS]);

impl Default for G1Points {
	fn default() -> Self {
		Self([[0; BYTES_PER_G1_POINT]; NUM_G1_POINTS])
	}
}

/// A newtype over list of G2 point from kzg trusted setup.
#[derive(Debug, Clone, Eq, PartialEq, AsRef, AsMut, Deref, DerefMut)]
#[repr(transparent)]
struct G2Points(pub [[u8; BYTES_PER_G2_POINT]; NUM_G2_POINTS]);

impl Default for G2Points {
	fn default() -> Self {
		Self([[0; BYTES_PER_G2_POINT]; NUM_G2_POINTS])
	}
}

/// Default G1 points.
const G1_POINTS: &G1Points = {
	const BYTES: &[u8] = include_bytes!("../assets/g1_points.bin");
	assert!(BYTES.len() == core::mem::size_of::<G1Points>());
	unsafe { &*BYTES.as_ptr().cast::<G1Points>() }
};

/// Default G2 points.
const G2_POINTS: &G2Points = {
	const BYTES: &[u8] = include_bytes!("../assets/g2_points.bin");
	assert!(BYTES.len() == core::mem::size_of::<G2Points>());
	unsafe { &*BYTES.as_ptr().cast::<G2Points>() }
};

/// Parses the contents of a KZG trusted setup file into a list of G1 and G2 points.
///
/// These can then be used to create a KZG settings object with
/// [`KzgSettings::load_trusted_setup`](c_kzg::KzgSettings::load_trusted_setup).
#[allow(dead_code)]
fn parse_kzg_trusted_setup(
	trusted_setup: &str,
) -> Result<(Box<G1Points>, Box<G2Points>), &'static str> {
	let mut lines = trusted_setup.lines();

	// load number of points
	let n_g1 = lines
		.next()
		.ok_or("KzgFileFormatError")?
		.parse::<usize>()
		.map_err(|_| "KzgParseError")?;
	let n_g2 = lines
		.next()
		.ok_or("KzgFileFormatError")?
		.parse::<usize>()
		.map_err(|_| "KzgParseError")?;

	if n_g1 != NUM_G1_POINTS {
		return Err("KzgMismatchedNumberOfPoints");
	}

	if n_g2 != NUM_G2_POINTS {
		return Err("KzgMismatchedNumberOfPoints");
	}

	// load g1 points
	let mut g1_points = Box::<G1Points>::default();
	for bytes in &mut g1_points.0 {
		let line = lines.next().ok_or("KzgFileFormatError")?;
		hex::decode_to_slice(line, bytes).map_err(|_| "KzgParseError")?;
	}

	// load g2 points
	let mut g2_points = Box::<G2Points>::default();
	for bytes in &mut g2_points.0 {
		let line = lines.next().ok_or("KzgFileFormatError")?;
		hex::decode_to_slice(line, bytes).map_err(|_| "KzgParseError")?;
	}

	if lines.next().is_some() {
		return Err("KzgFileFormatError");
	}

	Ok((g1_points, g2_points))
}

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub enum EnvKzgSettings {
	/// Default mainnet trusted setup
	#[default]
	Default,
	/// Custom trusted setup.
	Custom(Rc<KzgSettings>),
}

// Implement PartialEq and Hash manually because `c_kzg::KzgSettings` does not implement them
impl PartialEq for EnvKzgSettings {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Default, Self::Default) => true,
			(Self::Custom(a), Self::Custom(b)) => Rc::ptr_eq(a, b),
			_ => false,
		}
	}
}

impl Hash for EnvKzgSettings {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			Self::Default => {}
			Self::Custom(settings) => Rc::as_ptr(settings).hash(state),
		}
	}
}

impl EnvKzgSettings {
	/// Return set KZG settings.
	///
	/// In will initialize the default settings if it is not already loaded.
	pub fn get(&self) -> Rc<KzgSettings> {
		match self {
			Self::Default => {
				let res = KzgSettings::load_trusted_setup(G1_POINTS.as_ref(), G2_POINTS.as_ref())
					.expect("failed to load default trusted setup");
				Rc::new(res)
			}
			Self::Custom(settings) => settings.clone(),
		}
	}
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
#[inline]
pub fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
	const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;
	let mut hash: [u8; 32] = sha2::Sha256::digest(commitment).into();
	hash[0] = VERSIONED_HASH_VERSION_KZG;
	hash
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct KzgInput {
	commitment: Bytes48,
	z: Bytes32,
	y: Bytes32,
	proof: Bytes48,
}

impl KzgInput {
	#[inline]
	pub fn verify_kzg_proof(&self, kzg_settings: &KzgSettings) -> bool {
		KzgProof::verify_kzg_proof(
			&self.commitment,
			&self.z,
			&self.y,
			&self.proof,
			kzg_settings,
		)
		.unwrap_or(false)
	}
}

impl TryFrom<&[u8]> for KzgInput {
	type Error = &'static str;

	fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
		if input.len() != 192 {
			return Err("BlobInvalidInputLength");
		}
		// Verify commitment matches versioned_hash
		let versioned_hash = &input[..32];
		let commitment = &input[96..144];
		if kzg_to_versioned_hash(commitment) != versioned_hash {
			return Err("BlobMismatchedVersion");
		}
		let commitment = *as_bytes48(commitment);
		let z = *as_bytes32(&input[32..64]);
		let y = *as_bytes32(&input[64..96]);
		let proof = *as_bytes48(&input[144..192]);
		Ok(Self {
			commitment,
			z,
			y,
			proof,
		})
	}
}

#[inline]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
	bytes.try_into().expect("slice with incorrect length")
}

#[inline]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
	// SAFETY: `#[repr(C)] Bytes32([u8; 32])`
	unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

#[inline]
fn as_bytes48(bytes: &[u8]) -> &Bytes48 {
	// SAFETY: `#[repr(C)] Bytes48([u8; 48])`
	unsafe { &*as_array::<48>(bytes).as_ptr().cast() }
}

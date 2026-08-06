use std::{
	collections::{BTreeMap, BTreeSet},
	fs::File,
	io::BufReader,
	path::{Component, Path, PathBuf},
};

use rayon::prelude::*;

use crate::{
	execute::execute_case,
	schema::{FixtureFile, FixtureIndex, FixtureIndexEntry, ReleaseManifest, ReleaseTarget},
};

const RELEASE_MANIFEST: &str = include_str!("../fixtures-release.json");

#[derive(Clone, Debug)]
pub struct RunOptions {
	pub fixtures: PathBuf,
	pub target: String,
	pub cases: Vec<String>,
	pub jobs: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaseResult {
	Passed { id: String },
	Failed { id: String, detail: String },
}

impl CaseResult {
	pub fn id(&self) -> &str {
		match self {
			Self::Passed { id } | Self::Failed { id, .. } => id,
		}
	}

	pub fn passed(&self) -> bool {
		matches!(self, Self::Passed { .. })
	}
}

#[derive(Clone, Debug)]
pub struct RunSummary {
	pub target: String,
	pub selected: usize,
	pub passed: usize,
	pub failed: usize,
	pub results: Vec<CaseResult>,
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
	#[error("failed to parse embedded fixture release manifest: {0}")]
	Manifest(serde_json::Error),
	#[error("unknown target {target:?}; manifest targets: {available}")]
	UnknownTarget { target: String, available: String },
	#[error("invalid worker count 0")]
	ZeroJobs,
	#[error("failed to open {path}: {source}")]
	Open {
		path: PathBuf,
		source: std::io::Error,
	},
	#[error("failed to parse {path}: {source}")]
	Json {
		path: PathBuf,
		source: serde_json::Error,
	},
	#[error("fixture index root hash {actual} does not match pinned release root {expected}")]
	IndexRoot { actual: String, expected: String },
	#[error("fixture index selected no cases for target {0}")]
	NoCases(String),
	#[error("requested case IDs were not found: {0}")]
	MissingCases(String),
	#[error("target {target} selected {actual} cases; pinned release requires {expected}")]
	Count {
		target: String,
		actual: usize,
		expected: usize,
	},
	#[error("unsafe fixture index path {0:?}")]
	UnsafePath(String),
	#[error("fixture file {path} does not contain indexed case {id:?}")]
	MissingIndexedFixture { path: PathBuf, id: String },
	#[error("fixture hash for {id:?} is {actual:?}, index requires {expected:?}")]
	FixtureHash {
		id: String,
		actual: String,
		expected: String,
	},
	#[error("failed to create worker pool: {0}")]
	WorkerPool(String),
}

pub fn release_manifest() -> Result<ReleaseManifest, RunError> {
	serde_json::from_str(RELEASE_MANIFEST).map_err(RunError::Manifest)
}

pub fn run(options: &RunOptions) -> Result<RunSummary, RunError> {
	if options.jobs == Some(0) {
		return Err(RunError::ZeroJobs);
	}
	let manifest = release_manifest()?;
	let target = manifest
		.targets
		.get(&options.target)
		.ok_or_else(|| RunError::UnknownTarget {
			target: options.target.clone(),
			available: manifest
				.targets
				.keys()
				.cloned()
				.collect::<Vec<_>>()
				.join(", "),
		})?;
	let index_path = options.fixtures.join(".meta/index.json");
	let index: FixtureIndex = read_json(&index_path)?;
	if index.root_hash != manifest.index_root_hash {
		return Err(RunError::IndexRoot {
			actual: index.root_hash,
			expected: manifest.index_root_hash,
		});
	}
	let selected = select_entries(&index, &options.target, target, &options.cases)?;
	let groups = group_entries(selected);
	let execute = || {
		groups
			.par_iter()
			.map(|(path, entries)| run_file(&options.fixtures, path, entries, &options.target))
			.collect::<Result<Vec<Vec<CaseResult>>, RunError>>()
	};
	let chunks = match options.jobs {
		Some(jobs) => rayon::ThreadPoolBuilder::new()
			.num_threads(jobs)
			.build()
			.map_err(|error| RunError::WorkerPool(error.to_string()))?
			.install(execute)?,
		None => execute()?,
	};
	let mut results = chunks.into_iter().flatten().collect::<Vec<_>>();
	results.sort_by(|left, right| left.id().cmp(right.id()));
	let passed = results.iter().filter(|result| result.passed()).count();
	let failed = results.len() - passed;
	Ok(RunSummary {
		target: options.target.clone(),
		selected: results.len(),
		passed,
		failed,
		results,
	})
}

fn select_entries<'a>(
	index: &'a FixtureIndex,
	target_name: &str,
	target: &ReleaseTarget,
	case_ids: &[String],
) -> Result<Vec<&'a FixtureIndexEntry>, RunError> {
	let prefix = format!("{}/", target.path.trim_end_matches('/'));
	let requested = case_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
	let mut selected = index
		.test_cases
		.iter()
		.filter(|entry| {
			entry.fork == target_name
				&& entry.format == target.format
				&& entry.json_path.starts_with(&prefix)
				&& (requested.is_empty() || requested.contains(entry.id.as_str()))
		})
		.collect::<Vec<_>>();
	selected.sort_by(|left, right| left.id.cmp(&right.id));
	if !requested.is_empty() {
		let found = selected
			.iter()
			.map(|entry| entry.id.as_str())
			.collect::<BTreeSet<_>>();
		let missing = requested.difference(&found).copied().collect::<Vec<_>>();
		if !missing.is_empty() {
			return Err(RunError::MissingCases(missing.join(", ")));
		}
	} else if selected.is_empty() {
		return Err(RunError::NoCases(target_name.to_owned()));
	} else if selected.len() != target.expected_count {
		return Err(RunError::Count {
			target: target_name.to_owned(),
			actual: selected.len(),
			expected: target.expected_count,
		});
	}
	Ok(selected)
}

fn group_entries(entries: Vec<&FixtureIndexEntry>) -> BTreeMap<String, Vec<&FixtureIndexEntry>> {
	let mut groups = BTreeMap::<String, Vec<&FixtureIndexEntry>>::new();
	for entry in entries {
		groups
			.entry(entry.json_path.clone())
			.or_default()
			.push(entry);
	}
	groups
}

fn run_file(
	fixtures_root: &Path,
	json_path: &str,
	entries: &[&FixtureIndexEntry],
	target: &str,
) -> Result<Vec<CaseResult>, RunError> {
	let relative = Path::new(json_path);
	if relative.is_absolute()
		|| relative.components().any(|component| {
			matches!(
				component,
				Component::ParentDir | Component::RootDir | Component::Prefix(_)
			)
		}) {
		return Err(RunError::UnsafePath(json_path.to_owned()));
	}
	let path = fixtures_root.join(relative);
	let file: FixtureFile = read_json(&path)?;
	entries
		.iter()
		.map(|entry| {
			let fixture = file
				.get(&entry.id)
				.ok_or_else(|| RunError::MissingIndexedFixture {
					path: path.clone(),
					id: entry.id.clone(),
				})?;
			let actual_hash = fixture
				.info
				.get("hash")
				.and_then(serde_json::Value::as_str)
				.unwrap_or_default();
			if !equal_hex_integer(actual_hash, &entry.fixture_hash) {
				return Err(RunError::FixtureHash {
					id: entry.id.clone(),
					actual: actual_hash.to_owned(),
					expected: entry.fixture_hash.clone(),
				});
			}
			Ok(match execute_case(fixture, target) {
				Ok(()) => CaseResult::Passed {
					id: entry.id.clone(),
				},
				Err(error) => CaseResult::Failed {
					id: entry.id.clone(),
					detail: error.detail,
				},
			})
		})
		.collect()
}

fn equal_hex_integer(left: &str, right: &str) -> bool {
	fn normalized(value: &str) -> Option<&str> {
		let digits = value.strip_prefix("0x")?;
		if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
			return None;
		}
		let digits = digits.trim_start_matches('0');
		Some(if digits.is_empty() { "0" } else { digits })
	}
	match (normalized(left), normalized(right)) {
		(Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
		_ => false,
	}
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, RunError> {
	let file = File::open(path).map_err(|source| RunError::Open {
		path: path.to_owned(),
		source,
	})?;
	serde_json::from_reader(BufReader::new(file)).map_err(|source| RunError::Json {
		path: path.to_owned(),
		source,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	fn target() -> ReleaseTarget {
		ReleaseTarget {
			path: "state_tests/for_cancun".into(),
			format: "state_test".into(),
			expected_count: 2,
		}
	}

	fn index() -> FixtureIndex {
		FixtureIndex {
			root_hash: "root".into(),
			created_at: None,
			test_count: 3,
			test_cases: vec![
				FixtureIndexEntry {
					id: "case-b".into(),
					fixture_hash: "b".into(),
					fork: "Cancun".into(),
					format: "state_test".into(),
					json_path: "state_tests/for_cancun/b.json".into(),
				},
				FixtureIndexEntry {
					id: "case-a".into(),
					fixture_hash: "a".into(),
					fork: "Cancun".into(),
					format: "state_test".into(),
					json_path: "state_tests/for_cancun/a.json".into(),
				},
				FixtureIndexEntry {
					id: "wrong-format".into(),
					fixture_hash: "c".into(),
					fork: "Cancun".into(),
					format: "blockchain_test".into(),
					json_path: "state_tests/for_cancun/c.json".into(),
				},
			],
		}
	}

	#[test]
	fn selection_is_strict_and_sorted() {
		let index = index();
		let selected = select_entries(&index, "Cancun", &target(), &[]).unwrap();
		assert_eq!(
			selected
				.iter()
				.map(|entry| entry.id.as_str())
				.collect::<Vec<_>>(),
			["case-a", "case-b"]
		);
	}

	#[test]
	fn missing_exact_case_is_an_error() {
		let error = select_entries(&index(), "Cancun", &target(), &["absent".into()]).unwrap_err();
		assert!(matches!(error, RunError::MissingCases(_)));
	}

	#[test]
	fn embedded_release_manifest_is_valid() {
		let manifest = release_manifest().unwrap();
		assert_eq!(manifest.targets["Cancun"].expected_count, 11_554);
	}

	#[test]
	fn fixture_hashes_allow_omitted_leading_zeroes() {
		assert!(equal_hex_integer("0x03e17c", "0x3e17c"));
		assert!(!equal_hex_integer("0x03e17c", "0x4e17c"));
	}
}

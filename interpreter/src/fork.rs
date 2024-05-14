/// EVM supported forks
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fork {
	FRONTIER,
	ISTANBUL,
	BERLIN,
	LONDON,
	MERGE,
	SHANGHAI,
	CANCUN,
}

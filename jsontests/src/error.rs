use crate::types::TestExpectException;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum TestError {
	#[error("state root is different")]
	StateMismatch,
	#[error("expect error, but got okay")]
	ExpectException(TestExpectException),
	#[error("unexpected decoding error")]
	UnexpectedDecoding,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("io error")]
	IO(#[from] std::io::Error),
	#[error("json error")]
	JSON(#[from] serde_json::Error),
	#[error("evm error")]
	EVM(#[from] evm::interpreter::ExitError),
	#[error("unsupported fork")]
	UnsupportedFork,
	#[error("non-utf8 filename")]
	NonUtf8Filename,
	#[error("test error")]
	Test(#[from] TestError),
}

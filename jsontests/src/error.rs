#![allow(clippy::upper_case_acronyms)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
	#[error("state root is different")]
	StateMismatch,
	#[error("expect error, but got okay")]
	ExpectException,
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("io error")]
	IO(#[from] std::io::Error),
	#[error("json error")]
	JSON(#[from] serde_json::Error),
	#[error("evm error")]
	EVM(#[from] evm::ExitError),
	#[error("unsupported fork")]
	UnsupportedFork,
	#[error("non-utf8 filename")]
	NonUtf8Filename,
	#[error("test error")]
	Test(#[from] TestError),
}

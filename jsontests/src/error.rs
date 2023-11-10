use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
	#[error("state root is different")]
	StateMismatch,
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
	#[error("test error")]
	Test(#[from] TestError),
}

#[derive(Debug, Clone)]
pub enum MemoryError {
    IndexNotSupported,
}

#[derive(Debug, Clone)]
pub enum StackError {
    StackOverflow,
    StackUnderflow,
}

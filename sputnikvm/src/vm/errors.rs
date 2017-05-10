#[derive(Debug, Clone)]
pub enum MemoryError {
    IndexNotSupported,
}

#[derive(Debug, Clone)]
pub enum StackError {
    Overflow,
    Underflow,
}

#[derive(Debug, Clone)]
pub enum PCError {
    IndexNotSupported,
    BadJumpDest,
    Overflow,
}

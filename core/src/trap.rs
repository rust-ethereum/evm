use crate::ExternalOpcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Trap {
    Exit(ExitReason),
    External(ExternalOpcode),
    NotSupported,
}

impl From<ExitReason> for Trap {
    fn from(reason: ExitReason) -> Trap {
        Trap::Exit(reason)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
    CodeEnded,
    Returned,
    Reverted,
    StackUnderflow,
    StackOverflow,
    InvalidJump,
    PCUnderflow,

    OutOfGas,
    Other(&'static str),
}

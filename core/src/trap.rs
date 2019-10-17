use crate::ExternalOpcode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Trap {
    Exit(ExitReason),
    External(ExternalOpcode),
}

impl From<ExitReason> for Trap {
    fn from(reason: ExitReason) -> Trap {
        Trap::Exit(reason)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
    Stopped,
    Returned,
    Reverted,
    StackUnderflow,
    StackOverflow,
    InvalidJump,
    InvalidReturnRange,
    PCUnderflow,
    DesignatedInvalid,

    OutOfGas,
    NotSupported,
    Other(&'static str),
}

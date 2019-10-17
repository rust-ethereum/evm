use crate::ExternalOpcode;

pub enum Trap {
    Exit(ExitReason),
    External(ExternalOpcode),
    NotSupported,
}

pub enum ExitReason {
    CodeEnded,
    Returned,
    Reverted,
    OutOfGas,
    Other(&'static str),
}

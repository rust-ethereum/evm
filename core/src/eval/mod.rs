use crate::{ExitReason, Core, Opcode};

pub enum Control {
    Continue(usize),
    Exit(ExitReason),
    Jump(usize),
}

pub fn eval(opcode: Opcode, position: usize, state: &mut Core) -> Control {
    unimplemented!()
}

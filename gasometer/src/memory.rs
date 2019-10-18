use core::cmp::max;
use primitive_types::{U256, H256};
use evm_core::{Opcode, ExternalOpcode, ExitError, Stack};

fn memory_expand(
    current: usize,
    from: U256,
    len: U256,
) -> Result<usize, ExitError> {
    if len == U256::zero() {
        return Ok(current)
    }

    let end = from.checked_add(len).ok_or(ExitError::OutOfGas)?;

    if end > U256::from(usize::max_value()) {
        return Err(ExitError::OutOfGas)
    }
    let end = end.as_usize();

    let rem = end % 32;
    let new = if rem == 0 {
        end / 32
    } else {
        end / 32 + 1
    };
    Ok(max(current, new))
}

pub fn memory_cost(current: usize, opcode: Result<Opcode, ExternalOpcode>, stack: &Stack) -> Result<usize, ExitError> {
    match opcode {
        Err(ExternalOpcode::Sha3) | Ok(Opcode::Return) | Ok(Opcode::Revert) |
        Err(ExternalOpcode::Log(_)) =>
            memory_expand(current,
                          U256::from(&stack.peek(0)?[..]),
                          U256::from(&stack.peek(1)?[..])),
        Ok(Opcode::CodeCopy) | Ok(Opcode::CallDataCopy) | Err(ExternalOpcode::ReturnDataCopy) =>
            memory_expand(current,
                          U256::from(&stack.peek(0)?[..]),
                          U256::from(&stack.peek(2)?[..])),
        Err(ExternalOpcode::ExtCodeCopy) =>
            memory_expand(current,
                          U256::from(&stack.peek(1)?[..]),
                          U256::from(&stack.peek(3)?[..])),
        Ok(Opcode::MLoad) | Ok(Opcode::MStore) =>
            memory_expand(current,
                          U256::from(&stack.peek(0)?[..]),
                          U256::from(32)),
        Ok(Opcode::MStore8) =>
            memory_expand(current,
                          U256::from(&stack.peek(0)?[..]),
                          U256::one()),
        Err(ExternalOpcode::Create) | Err(ExternalOpcode::Create2) =>
            memory_expand(current,
                          U256::from(&stack.peek(1)?[..]),
                          U256::from(&stack.peek(2)?[..])),
        Err(ExternalOpcode::Call) | Err(ExternalOpcode::CallCode) =>
            memory_expand(
                memory_expand(current,
                              U256::from(&stack.peek(3)?[..]),
                              U256::from(&stack.peek(4)?[..]))?,
                U256::from(&stack.peek(5)?[..]),
                U256::from(&stack.peek(6)?[..])),
        _ => Ok(current),
    }
}

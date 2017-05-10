use utils::bigint::M256;
use vm::{Storage, Memory};
use vm::eval::State;
use vm::errors::MachineError;

pub fn check_range(start: M256, len: M256) -> Result<(), MachineError> {
    if start + len < start {
        Err(MachineError::InvalidRange)
    } else {
        Ok(())
    }
}

pub fn copy_from<M: Memory, S: Storage>(state: &State<M, S>, start: M256, len: M256) -> Result<Vec<u8>, MachineError> {
    check_range(start, len)?;

    let mut result: Vec<u8> = Vec::new();
    let mut i = start;
    while i < start + len {
        result.push(state.memory.read_raw(i));
        i = i + M256::from(1u64);
    }

    Ok(result)
}

pub fn copy_into<M: Memory, S: Storage>(state: &mut State<M, S>, values: &[u8], start: M256, len: M256) -> Result<(), MachineError> {
    check_range(start, len)?;
    let mut i = start;
    while i < start + len {
        state.memory.check_write(i)?;
        i = i + M256::from(1u64);
    }

    let mut i = start;
    let mut j = 0;
    while i < start + len {
        if j < values.len() {
            state.memory.write_raw(i, values[j]).unwrap();
            j = j + 1;
        } else {
            state.memory.write_raw(i, 0u8).unwrap();
        }
        i = i + M256::from(1u64);
    }
    Ok(())
}

fn call_cost(
    opcode: ExternalOpcode,
    stack: &Stack,
    new_account: bool,
    config: &GasometerConfig,
) -> Result<usize, ExitError> {
    let transfers_value = stack.peek(2)? != H256::default();
    config.gas_call +
        xfer_cost(opcode, transfers_value) +
        new_cost(instruction, stack, transfers_value)
}

fn xfer_cost(
    opcode: ExternalOpcode,
    transfers_value: bool
) -> usize {
    if (instruction == &ExternalOpcode::Call || instruction == &ExternalOpcode::CallCode) &&
        transfers_value
    {
        G_CALLVALUE
    } else {
        0
    }
}

fn new_cost(
    opcode: ExternalOpcode,
    stack: &Stack,
    new_account: bool,
    transfers_value: bool,
    config: &GasometerConfig,
) -> Option<usize> {
    let eip161 = config.empty_considered_exists;
    if instruction == &Instruction::CALL || instruction == &Instruction::STATICCALL {
        if eip161 {
            if transfers_value && new_account {
                Gas::from(G_NEWACCOUNT)
            } else {
                Gas::zero()
            }
        } else if new_account {
            Gas::from(G_NEWACCOUNT)
        } else {
            Gas::zero()
        }
    } else {
        Gas::zero()
    }
}

pub fn gas_cost(
    opcode: Result<Opcode, ExternalOpcode>,
    stack: &Stack,
    storage_access: Option<&(H256, H256)>
) -> Result<usize, ExitError> {
    match opcode {

    }
}

use super::{Memory, VectorMemory, Stack, VectorStack, PC, Result, Gas};
use ::account::Account;

pub type VectorMachine<'p> = Machine<'p, VectorMemory, VectorStack>;

pub struct Machine<'p, M, S> {
    available_gas: Gas,
    pub pc: PC<'p>,
    pub memory: M, // Contains i
    pub stack: S,
    pub account: Account,
}

impl<'p, M, S> Machine<'p, M, S> where M: Memory, S: Stack {
    pub fn new(code: &'p [u8], data: &[u8], available_gas: Gas) -> Self {
        Self {
            available_gas: available_gas,
            pc: PC::new(code),
            memory: M::new(),
            stack: S::new(),
            account: Account::default(),
        }
    }

    pub fn available_gas(&self) -> Gas {
        self.available_gas
    }

    pub fn step(&mut self) -> bool {
        if self.pc.is_stopped() || !self.available_gas.is_valid() {
            return false;
        }

        let opcode = self.pc.read_opcode();
        self.available_gas -= opcode.gas_cost_before(self);
        opcode.run(self);
        self.available_gas -= opcode.gas_cost_after(self);

        true
    }

    pub fn fire(&mut self) -> Result<Gas> {
        while self.step() { }
        Ok(self.available_gas)
    }
}

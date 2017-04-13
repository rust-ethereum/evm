use super::{Memory, VectorMemory, Stack, VectorStack, PC, Result, Gas};
use account::{Account, FakeAccount};

pub type VectorMachine<'p> = Machine<'p, VectorMemory, VectorStack, FakeAccount>;

pub trait Machine {
    type P: PC;
    type M: Memory;
    type Sta: Stack;
    type Sto: Storage;
    type T: Transaction;
    type B: Block;
    type Bc: Blockchain;

    fn pc(&self) -> &P;
    fn pc_mut(&mut self) -> &mut P;
    fn memory(&self) -> &M;
    fn memory_mut(&mut self) -> &mut M;
    fn stack(&self) -> &Sta;
    fn stack_mut(&mut self) -> &mut Sta;
    fn storage(&self) -> &Sto;
    fn storage_mut(&mut self) -> &mut Sto;

    fn transaction(&self) -> &T;
    fn block(&self) -> &B;
    fn blockchain(&self) -> &Bc;
    fn available_gas(&self) -> Gas;
}

pub struct VectorMachine {
    pc: VectorPC,
    memory: VectorMemory,
    stack: VectorStack,
    storage: FakeStorage,
    transaction: FakeTransaction,
    block: FakeBlock,
    blockchain: FakeBlockchain,
    available_gas: Gas,
}

impl VectorMachine {
    pub fn new(code: &[u8], data: &[u8], available_gas: Gas) -> VectorMachine {
        VectorMachine {
            pc: VectorPC::new(code),
            memory: VectorMemory::new(),
            stack: VectorStack::new(),
            storage: FakeStorage::new(),
            transaction: FakeTransaction::message_call(U256::zero(), data),
            block: FakeBlock,
            blockchain: FakeBlockchain,
            available_gas: available_gas,
        }
    }
}

impl Machine for VectorMachine {
    type P = VectorPC;
    type M = VectorMemory;
    type Sta = VectorStack;
    type Sto = FakeStorage;
    type T = FakeTransaction;
    type B = FakeBlock;
    type Bc = FakeBlockchain;

    fn pc(&self) -> &P {
        &self.pc
    }

    fn pc_mut(&mut self) -> &mut P {
        &mut self.pc
    }

    fn memory(&self) -> &M {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    fn stack(&self) -> &Sta {
        &self.stack
    }

    fn stack_mut(&mut self) -> &mut Sta {
        &mut self.stack
    }

    fn storage(&self) -> &Sto
}

impl<'p, M, S, A> Machine<'p, M, S, A> where M: Memory, S: Stack, A: Account {
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

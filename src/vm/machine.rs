use utils::u256::U256;
use utils::gas::Gas;
use super::{Memory, VectorMemory, Stack, VectorStack, PC, VectorPC, Result};
use account::{Storage, VectorStorage};
use blockchain::{Block, Blockchain, FakeBlock, FakeBlockchain};
use transaction::{Transaction, VectorTransaction};

pub trait Machine {
    type P: PC;
    type M: Memory;
    type Sta: Stack;
    type Sto: Storage;
    type T: Transaction;
    type B: Block;
    type Bc: Blockchain;

    fn pc(&self) -> &Self::P;
    fn pc_mut(&mut self) -> &mut Self::P;
    fn memory(&self) -> &Self::M;
    fn memory_mut(&mut self) -> &mut Self::M;
    fn stack(&self) -> &Self::Sta;
    fn stack_mut(&mut self) -> &mut Self::Sta;
    fn storage(&self) -> &Self::Sto;
    fn storage_mut(&mut self) -> &mut Self::Sto;

    fn transaction(&self) -> &Self::T;
    fn block(&self) -> &Self::B;
    fn blockchain(&self) -> &Self::Bc;

    fn use_gas(&mut self, gas: Gas);
    fn used_gas(&self) -> Gas;

    fn step(&mut self) -> bool where Self: Sized {
        if self.pc().stopped() || !self.available_gas().is_valid() {
            return false;
        }

        let opcode = self.pc_mut().read_opcode();
        let before = opcode.gas_cost_before(self);
        self.use_gas(before);
        opcode.run(self);
        let after = opcode.gas_cost_after(self);
        self.use_gas(after);

        true
    }

    fn fire(&mut self) where Self: Sized {
        while self.step() { }
    }

    fn available_gas(&self) -> Gas {
        self.transaction().gas_limit() - self.used_gas()
    }
}

pub struct VectorMachine<B, Bc> {
    pc: VectorPC,
    memory: VectorMemory,
    stack: VectorStack,
    storage: VectorStorage,
    transaction: VectorTransaction,
    block: B,
    blockchain: Bc,
    used_gas: Gas,
}

pub type FakeVectorMachine = VectorMachine<FakeBlock, FakeBlockchain>;

impl FakeVectorMachine {
    pub fn new(code: &[u8], data: &[u8], gas_limit: Gas) -> FakeVectorMachine {
        VectorMachine {
            pc: VectorPC::new(code),
            memory: VectorMemory::new(),
            stack: VectorStack::new(),
            storage: VectorStorage::new(),
            transaction: VectorTransaction::message_call(U256::zero(), data, gas_limit),
            block: FakeBlock,
            blockchain: FakeBlockchain,
            used_gas: Gas::zero(),
        }
    }
}

impl<B0: Block, Bc0: Blockchain> Machine for VectorMachine<B0, Bc0> {
    type P = VectorPC;
    type M = VectorMemory;
    type Sta = VectorStack;
    type Sto = VectorStorage;
    type T = VectorTransaction;
    type B = B0;
    type Bc = Bc0;

    fn use_gas(&mut self, gas: Gas) {
        self.used_gas += gas;
    }

    fn used_gas(&self) -> Gas {
        self.used_gas
    }

    fn pc(&self) -> &Self::P {
        &self.pc
    }

    fn pc_mut(&mut self) -> &mut Self::P {
        &mut self.pc
    }

    fn memory(&self) -> &Self::M {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut Self::M {
        &mut self.memory
    }

    fn stack(&self) -> &Self::Sta {
        &self.stack
    }

    fn stack_mut(&mut self) -> &mut Self::Sta {
        &mut self.stack
    }

    fn storage(&self) -> &Self::Sto {
        &self.storage
    }

    fn storage_mut(&mut self) -> &mut Self::Sto {
        &mut self.storage
    }

    fn transaction(&self) -> &Self::T {
        &self.transaction
    }

    fn block(&self) -> &Self::B {
        &self.block
    }

    fn blockchain(&self) -> &Self::Bc {
        &self.blockchain
    }
}

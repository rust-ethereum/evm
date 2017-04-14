use utils::u256::U256;
use utils::gas::Gas;
use utils::address::Address;

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
    fn block_mut(&mut self) -> &mut Self::B;
    fn blockchain(&self) -> &Self::Bc;

    fn use_gas(&mut self, gas: Gas);
    fn used_gas(&self) -> Gas;
    fn return_values(&self) -> &[u8];
    fn set_return_values(&mut self, data: &[u8]);

    fn fork<F: FnOnce(&mut Self)>(&mut self, gas: Gas, from: Address,
                                  to: Address, value: U256,
                                  memory_in_start: U256,
                                  memory_in_len: U256,
                                  memory_out_start: U256,
                                  memory_out_len: U256, f: F);

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

pub struct VectorMachine<'a, B: 'a, Bc: 'a> {
    pc: VectorPC,
    memory: VectorMemory,
    stack: VectorStack,
    storage: VectorStorage,
    transaction: VectorTransaction,
    return_values: Vec<u8>,
    block: &'a mut B,
    blockchain: &'a Bc,
    used_gas: Gas,
}

pub type FakeVectorMachine = VectorMachine<'static, FakeBlock, FakeBlockchain>;

static mut FAKE_BLOCK: FakeBlock = FakeBlock;
static FAKE_BLOCKCHAIN: FakeBlockchain = FakeBlockchain;

impl FakeVectorMachine {
    pub fn new(code: &[u8], data: &[u8], gas_limit: Gas) -> FakeVectorMachine {
        VectorMachine {
            pc: VectorPC::new(code),
            memory: VectorMemory::new(),
            stack: VectorStack::new(),
            storage: VectorStorage::new(),
            transaction: VectorTransaction::message_call(Address::default(), Address::default(),
                                                         U256::zero(), data, gas_limit),
            return_values: Vec::new(),
            block: unsafe { &mut FAKE_BLOCK }, // FakeBlock doesn't contain any field. So this unsafe is okay.
            blockchain: &FAKE_BLOCKCHAIN,
            used_gas: Gas::zero(),
        }
    }
}

impl<'a, B0: Block + 'a, Bc0: Blockchain + 'a> Machine for VectorMachine<'a, B0, Bc0> {
    type P = VectorPC;
    type M = VectorMemory;
    type Sta = VectorStack;
    type Sto = VectorStorage;
    type T = VectorTransaction;
    type B = B0;
    type Bc = Bc0;

    fn return_values(&self) -> &[u8] {
        self.return_values.as_ref()
    }

    fn set_return_values(&mut self, val: &[u8]) {
        self.return_values = val.into();
    }

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

    fn block_mut(&mut self) -> &mut Self::B {
        &mut self.block
    }

    fn blockchain(&self) -> &Self::Bc {
        &self.blockchain
    }

    fn fork<F: FnOnce(&mut Self)>(&mut self, gas: Gas, from: Address, to: Address,
                                  value: U256,
                                  memory_in_start: U256, memory_in_len: U256,
                                  memory_out_start: U256, memory_out_len: U256,
                                  f: F) {
        use std::mem::swap;

        let from = from;
        let storage_data: Vec<U256> = self.storage().as_ref().into();
        self.block_mut().set_account_storage(from, storage_data.as_ref());
        let mem_in_start: usize = memory_in_start.into();
        let mem_in_len: usize = memory_in_len.into();
        let mem_in_end: usize = mem_in_start + mem_in_len;
        let mem_in: Vec<u8> = self.memory().as_ref()[mem_in_start..mem_in_end].into();

        let mut new_transaction = VectorTransaction::message_call(from, to, value, mem_in.as_ref(), gas);
        // TODO: register this transaction to the block.
        let mut new_pc = VectorPC::new(if to == from { self.pc().code() }
                               else { self.block().account_code(to).unwrap() });
        let mut new_stack = VectorStack::new();
        let mut new_memory = VectorMemory::new();
        let mut new_storage = VectorStorage::with_storage(if to == from { self.storage().as_ref() }
                                                          else { self.block().account_storage(to) });
        let mut new_return_values: Vec<u8> = Vec::new();
        let mut new_used_gas = Gas::zero();

        swap(&mut new_transaction, &mut self.transaction);
        swap(&mut new_pc, &mut self.pc);
        swap(&mut new_stack, &mut self.stack);
        swap(&mut new_memory, &mut self.memory);
        swap(&mut new_storage, &mut self.storage);
        swap(&mut new_return_values, &mut self.return_values);
        swap(&mut new_used_gas, &mut self.used_gas);

        f(self);

        swap(&mut new_transaction, &mut self.transaction);
        swap(&mut new_pc, &mut self.pc);
        swap(&mut new_stack, &mut self.stack);
        swap(&mut new_memory, &mut self.memory);
        swap(&mut new_storage, &mut self.storage);
        swap(&mut new_return_values, &mut self.return_values);
        swap(&mut new_used_gas, &mut self.used_gas);

        let mem_out_start: usize = memory_out_start.into();
        let mem_out_len: usize = memory_out_len.into();
        let mem_out_end: usize = mem_out_start + mem_out_len;

        for i in 0..mem_out_end {
            self.memory_mut().write_raw(memory_out_start + i.into(), new_return_values[i]);
        }
    }
}

use utils::bigint::{U256, M256};
use utils::gas::Gas;
use utils::address::Address;

use super::{Memory, VectorMemory, Stack, VectorStack, PC, VectorPC, Result, Error};
use super::cost::{gas_cost, CostAggregrator};
use blockchain::{Block, FakeVectorBlock};
use transaction::{Transaction, VectorTransaction};

use std::borrow::BorrowMut;
use std::marker::PhantomData;

pub trait MachineState {
    type P: PC;
    type M: Memory;
    type Sta: Stack;
    type T: Transaction;
    type B: Block;
    type Sub: MachineState;

    fn pc(&self) -> &Self::P;
    fn pc_mut(&mut self) -> &mut Self::P;
    fn memory(&self) -> &Self::M;
    fn memory_mut(&mut self) -> &mut Self::M;
    fn stack(&self) -> &Self::Sta;
    fn stack_mut(&mut self) -> &mut Self::Sta;

    fn transaction(&self) -> &Self::T;
    fn block(&self) -> &Self::B;
    fn block_mut(&mut self) -> &mut Self::B;

    fn return_values(&self) -> &[u8];
    fn set_return_values(&mut self, data: &[u8]);

    fn fork<R, F: FnOnce(Self::Sub) -> (R, Self::Sub)>(&mut self, gas: Gas, from: Address, to: Address,
                                                       value: M256, data: &[u8], code: &[u8], f: F) -> R;
}

pub struct VectorMachineState<B0, BR> {
    pc: VectorPC,
    memory: VectorMemory,
    stack: VectorStack,
    transaction: VectorTransaction,
    return_values: Vec<u8>,
    block: Option<BR>,
    _block_marker: PhantomData<B0>,
}

impl<B0: Block, BR: AsRef<B0> + AsMut<B0>> VectorMachineState<B0, BR> {
    pub fn new(code: &[u8], data: &[u8], gas_limit: Gas,
               transaction: VectorTransaction, block: BR) -> Self {
        VectorMachineState {
            pc: VectorPC::new(code),
            memory: VectorMemory::new(),
            stack: VectorStack::new(),
            transaction: transaction,
            return_values: Vec::new(),
            block: Some(block),
            _block_marker: PhantomData,
        }
    }
}

impl<B0: Block, BR: AsRef<B0> + AsMut<B0>> MachineState for VectorMachineState<B0, BR> {
    type P = VectorPC;
    type M = VectorMemory;
    type Sta = VectorStack;
    type T = VectorTransaction;
    type B = B0;
    type Sub = Self;

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

    fn transaction(&self) -> &Self::T {
        &self.transaction
    }

    fn block(&self) -> &Self::B {
        self.block.as_ref().unwrap().as_ref()
    }

    fn block_mut(&mut self) -> &mut Self::B {
        self.block.as_mut().unwrap().as_mut()
    }

    fn return_values(&self) -> &[u8] {
        self.return_values.as_ref()
    }

    fn set_return_values(&mut self, val: &[u8]) {
        self.return_values = val.into();
    }

    fn fork<R, F: FnOnce(Self::Sub) -> (R, Self::Sub)>(&mut self, gas: Gas, from: Address, to: Address,
                                                       value: M256, data: &[u8], code: &[u8], f: F) -> R {
        let mut state = Self {
            pc: VectorPC::new(code),
            memory: VectorMemory::new(),
            stack: VectorStack::new(),
            transaction: VectorTransaction::message_call(from, to, value, data, gas),
            return_values: Vec::new(),
            block: self.block.take(),
            _block_marker: PhantomData,
        };

        let (ret, mut state) = f(state);
        self.block = state.block.take();
        ret
    }
}

pub struct Machine<S> {
    state: S,
    cost_aggregrator: CostAggregrator,
    used_gas: Gas,
}

impl<S: MachineState> Machine<S> {
    pub fn from_state(state: S) -> Self {
        Machine {
            state: state,
            cost_aggregrator: CostAggregrator::default(),
            used_gas: Gas::zero()
        }
    }

    pub fn into_state(self) -> S {
        self.state
    }

    pub fn pc(&self) -> &S::P {
        self.state.pc()
    }

    pub fn memory(&self) -> &S::M {
        self.state.memory()
    }

    pub fn stack(&self) -> &S::Sta {
        self.state.stack()
    }

    pub fn transaction(&self) -> &S::T {
        self.state.transaction()
    }

    pub fn block(&self) -> &S::B {
        self.state.block()
    }

    pub fn return_values(&self) -> &[u8] {
        self.state.return_values()
    }

    pub fn peek_cost(&self) -> Result<Gas> {
        let opcode = self.pc().peek_opcode()?;
        let (gas, agg) = gas_cost(opcode, &self.state, self.available_gas(), self.cost_aggregrator)?;
        Ok(gas)
    }

    pub fn step(&mut self) -> Result<()> {
        // Constraints and assumptions for when the VM is running
        debug_assert!(self.transaction().data().is_some());
        debug_assert!(self.transaction().gas_price() <= Gas::from(U256::max_value()));

        begin_rescuable!(self, &mut Self, __);
        if self.pc().stopped() {
            trr!(Err(Error::Stopped), __);
        }

        let position = self.pc().position();
        on_rescue!(|machine| {
            machine.state.pc_mut().jump(position);
        }, __);

        let opcode = trr!(self.state.pc_mut().read_opcode(), __);
        let available_gas = self.available_gas();
        let cost_aggregrator = self.cost_aggregrator;
        let (gas, agg) = trr!(gas_cost(opcode, &mut self.state,
                                       available_gas, cost_aggregrator), __);

        if gas > self.available_gas() {
            trr!(Err(Error::EmptyGas), __);
        }

        trr!(opcode.run(&mut self.state, self.cost_aggregrator.active_memory_len()), __);

        self.cost_aggregrator = agg;
        self.used_gas = self.used_gas + gas;

        end_rescuable!(__);
        Ok(())
    }

    pub fn fire(&mut self) -> Result<()> {
        loop {
            let result = self.step();

            if result.is_err() {
                match result.err().unwrap() {
                    Error::Stopped => return Ok(()),
                    err => return Err(err),
                }
            }
        }
    }

    pub fn available_gas(&self) -> Gas {
        self.transaction().gas_limit() - self.used_gas
    }
}

pub type VectorMachine<B0, BR> = Machine<VectorMachineState<B0, BR>>;
pub type FakeVectorMachine = VectorMachine<FakeVectorBlock, Box<FakeVectorBlock>>;

impl<B0: Block, BR: AsRef<B0> + AsMut<B0>> VectorMachine<B0, BR> {
    pub fn new(code: &[u8], data: &[u8], gas_limit: Gas,
               transaction: VectorTransaction, block: BR) -> Self {
        VectorMachine::from_state(VectorMachineState::new(code, data, gas_limit,
                                                    transaction, block))
    }
}

impl FakeVectorMachine {
    pub fn fake(code: &[u8], data: &[u8], gas_limit: Gas) -> FakeVectorMachine {
        VectorMachine::new(code, data, gas_limit,
                           VectorTransaction::message_call(Address::default(), Address::default(),
                                                           M256::zero(), data, gas_limit),
                           Box::new(FakeVectorBlock::new()))
    }
}

use utils::u256::U256;

pub trait Stack {
    fn new() -> Self;
    fn push(&mut self, elem: U256);
    fn pop(&mut self) -> U256;
    fn set(&mut self, no_from_top: usize, val: U256);
    fn peek(&self, no_from_top: usize) -> &U256;
    fn has(&self, no_of_elems: usize) -> bool;
    fn size(&self) -> usize;
}

pub struct VectorStack {
    stack: Vec<U256>,
}

impl Stack for VectorStack {
    fn new() -> VectorStack {
        VectorStack {
            stack: Vec::new(),
        }
    }

    fn push(&mut self, elem: U256) {
        self.stack.push(elem);
    }

    fn pop(&mut self) -> U256 {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Empty stack pop.")
        }
    }

    fn set(&mut self, no_from_top: usize, val: U256) {
        self.stack[self.stack.len() - no_from_top - 1] = val;
    }

    fn peek(&self, no_from_top: usize) -> &U256 {
        &self.stack[self.stack.len() - no_from_top - 1]
    }

    fn has(&self, no_of_elems: usize) -> bool {
        self.stack.len() >= no_of_elems
    }

    fn size(&self) -> usize {
        self.stack.len()
    }
}

use utils::bigint::M256;

pub trait Stack {
    fn push(&mut self, elem: M256);
    fn pop(&mut self) -> M256;
    fn set(&mut self, no_from_top: usize, val: M256);
    fn peek(&self, no_from_top: usize) -> M256;
    fn has(&self, no_of_elems: usize) -> bool;
    fn size(&self) -> usize;
}

pub struct VectorStack {
    stack: Vec<M256>,
}

impl VectorStack {
    pub fn new() -> VectorStack {
        VectorStack {
            stack: Vec::new(),
        }
    }
}

impl Stack for VectorStack {
    fn push(&mut self, elem: M256) {
        self.stack.push(elem);
    }

    fn pop(&mut self) -> M256 {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Empty stack pop.")
        }
    }

    fn set(&mut self, no_from_top: usize, val: M256) {
        let len = self.stack.len();
        self.stack[len - no_from_top - 1] = val;
    }

    fn peek(&self, no_from_top: usize) -> M256 {
        self.stack[self.stack.len() - no_from_top - 1]
    }

    fn has(&self, no_of_elems: usize) -> bool {
        self.stack.len() >= no_of_elems
    }

    fn size(&self) -> usize {
        self.stack.len()
    }
}

use sputnikvm::Opcode;
use std::collections::{HashMap, hash_map};
use flame::Span;

pub struct Profiler {
    occurances: HashMap<Opcode, (usize, f64)>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            occurances: Default::default(),
        }
    }
}

impl Profiler {
    pub fn record(&mut self, opcode: Opcode, span: &Span) {
        match self.occurances.entry(opcode) {
            hash_map::Entry::Occupied(mut entry) => {
                let (occ, avg) = entry.get().clone();
                let (nocc, navg) = (occ + 1, ((avg * (occ as f64)) + (span.delta as f64)) / ((occ + 1) as f64));
                entry.insert((nocc, navg));
            },
            hash_map::Entry::Vacant(entry) => {
                entry.insert((1, span.delta as f64));
            },
        }
    }
}

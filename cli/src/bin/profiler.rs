use flame::Span;
use std::cmp::Ordering;
use std::collections::{hash_map, HashMap};

pub struct Profiler {
    occurances: HashMap<String, (usize, f64)>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            occurances: Default::default(),
        }
    }
}

impl Profiler {
    pub fn record(&mut self, span: &Span) {
        let opcode = span.name.to_string();
        match self.occurances.entry(opcode) {
            hash_map::Entry::Occupied(mut entry) => {
                let (occ, avg) = *entry.get();
                let (nocc, navg) = (
                    occ + 1,
                    ((avg * (occ as f64)) + (span.delta as f64)) / ((occ + 1) as f64),
                );
                entry.insert((nocc, navg));
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert((1, span.delta as f64));
            }
        }
    }

    pub fn print_stats(&self) {
        println!("--- Profiler Stats ---");
        let mut occs: Vec<_> = self.occurances.iter().collect();
        occs.sort_by(|&(_k1, v1), &(_k2, v2)| match v1.1.partial_cmp(&v2.1) {
            Some(val) => val,
            None => Ordering::Equal,
        });
        occs.reverse();
        for occ in occs {
            println!("{}: {:.0} ns ({} times)", occ.0, (occ.1).1, (occ.1).0);
        }
        println!("--- End Profiler Stats ---");
    }
}

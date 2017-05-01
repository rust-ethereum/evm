use rocksdb::{DB, Direction, IteratorMode};

pub fn perform_regression(path: &str) -> bool {
    let mut db = DB::open_default(path).unwrap();
    print!("HELLO REGRESSED WORLD!");
    false
}

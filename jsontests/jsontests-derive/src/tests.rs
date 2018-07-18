use std::path::Path;
use std::fs::{self, File, DirEntry};
use failure::Error;

use json::Value;
use json;

pub struct Test {
    pub path: String,
    pub name: String,
    pub data: Value
}

pub fn read_tests_from_dir<P: AsRef<Path>>(dir_path: P) -> Result<impl Iterator<Item=Test>, Error> {
    let dir = fs::read_dir(dir_path)?;

    let iter = dir.into_iter()
        .flat_map(|file|{
            match file {
                Ok(file) => tests_iterator_from_direntry(file).unwrap(),
                Err(err) => panic!("failed to read dir: {}", err)
            }
        });

    Ok(iter)
}

pub fn tests_iterator_from_direntry(file: DirEntry) -> Result<impl Iterator<Item=Test>, Error> {
    let path = file.path().to_owned();
    let file = File::open(&path)?;
    let tests: Value = json::from_reader(file)?;

    // Move out the root object
    let tests = match tests {
        Value::Object(tests) => tests,
        _ => panic!("expected a json object at the root of test file")
    };

    let iter = tests.into_iter().map(move |(name, data)| Test {
        path: path.to_str().unwrap().to_owned(),
        name,
        data
    });

    Ok(iter)
}

use std::path::Path;
use std::iter;
use std::fs::{self, File, DirEntry};
use failure::Error;

use serde_json as json;
use json::Value;

pub struct Test {
    pub path: String,
    pub name: String,
    pub data: Value
}

pub fn read_tests_from_dir<P: AsRef<Path>>(dir_path: P) -> Result<impl Iterator<Item=Test>, Error> {
    let dir = fs::read_dir(dir_path)?;

    let iter = dir
        .flat_map(|file|{
            match file {
                Ok(file) => tests_iterator_from_direntry(&file).unwrap(),
                Err(err) => panic!("failed to read dir: {}", err)
            }
        });

    Ok(iter)
}

pub fn tests_iterator_from_direntry(file: &DirEntry) -> Result<Box<dyn Iterator<Item=Test>>, Error> {
    let path = file.path().to_owned();

    // Skip non-json files
    if !path.extension().map(|e| e == "json").unwrap_or(false) {
        return Ok(Box::new(iter::empty()))
    }

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

    Ok(Box::new(iter))
}

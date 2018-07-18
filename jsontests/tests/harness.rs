#![allow(non_snake_case)]
#![allow(unused)]

#[macro_use]
extern crate jsontests_derive;
extern crate jsontests;

#[derive(JsonTests)]
#[directory = "jsontests/res/files/HarnessCorrectnessTests"]
#[test_with = "jsontests::util::run_test"]
#[should_panic]
struct HarnessCorrectness;

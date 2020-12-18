#![allow(unused)]

#[macro_use] extern crate html5ever;
#[macro_use] extern crate markup5ever;
#[macro_use] extern crate lazy_static;

pub mod frontend;
pub mod data;
pub mod parser;
pub mod macros;

fn main() {
    frontend::main();
}

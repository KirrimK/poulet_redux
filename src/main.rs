use std::io;

mod pouletlib;
mod cli;

use crate::pouletlib::logic;

fn main() {
    println!("Enter logic proposition using RPN:");
    let mut buffer = String::new();
    let stdin = io::stdin();
    let _ = stdin
        .read_line(&mut buffer)
        .expect("An error occured while reading user input");
    match logic::Prop::parse_rpn(&buffer) {
        Err(s) => println!("Could not parse string: {s}"),
        Ok(prop) => println!(
            "{} (depth: {}; items: {}) rewritten in RPN: {}",
            prop.to_string(),
            prop.depth(),
            prop.items(),
            prop.to_string_rpn()
        ),
    }
}

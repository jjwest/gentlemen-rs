#![feature(nll)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use failure::err_msg;

use std::env;
use std::fs::File;
use std::io::Read;

#[macro_use]
mod macros;
mod ast;
mod builtins;
mod parser;
mod lexer;

use lexer::Lexer;

fn main() {
    pretty_env_logger::init().unwrap();

    if let Err(e) = parse_args().and_then(|src| run(src)) {
        eprintln!("error: {}", e);
    }
}

fn run(source: Vec<u8>) -> Result<(), failure::Error> {
    let lexer = Lexer::new(source);
    let tokens = lexer.generate_tokens()?;
    let program = parser::parse(tokens)?;

    Ok(())
}

fn parse_args() -> Result<Vec<u8>, failure::Error> {
    let file_name = env::args()
        .nth(1)
        .ok_or_else(|| err_msg("Missing file name"))?;
    let mut file = File::open(&file_name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

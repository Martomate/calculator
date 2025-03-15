use std::io::{stdin, stdout};

mod cli;
mod parser;
mod repr;

fn main() {
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    cli::run_cli(&mut stdin, &mut stdout).unwrap()
}

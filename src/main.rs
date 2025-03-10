use std::io::{stdin, stdout, Write};

use colored::Colorize;

mod parser;
mod repr;

fn main() {
    let mut line = String::new();
    
    loop {
        print!("> ");
        stdout().flush().unwrap();
        
        line.clear();
        stdin().read_line(&mut line).unwrap();
        let line = line.strip_suffix('\n').unwrap_or_else(|| &line);

        match parser::parse_line(line) {
            Ok(v) => 
                match v.evaluate() {
                    Ok(res) => println!("{}", res.to_string().green()),
                    Err(err) => println!("{}", err.red()),
                }
            Err(err) => println!("{}", err.red()),
        }
    }
}

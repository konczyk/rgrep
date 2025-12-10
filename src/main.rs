use std::env;
use std::io;
use std::process;

fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Missing first argument to be '-E'");
        process::exit(1);
    }

    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    process::exit(0)
}

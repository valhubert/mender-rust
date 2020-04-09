use std::env;
use std::error::Error;
use std::process;

mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = parse::Command::new(&args).unwrap_or_else(|err| {
        println!("Parse error: {}", err);
        process::exit(1);
    });
    let config = parse::Config::new(command).unwrap_or_else(|err| {
        println!("Config error: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Run error: {}", e);
        process::exit(2);
    }
}

fn run(config: parse::Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}

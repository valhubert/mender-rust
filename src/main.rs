use std::env;
use std::process;
use std::error::Error;

mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = parse::Config::new(&args).unwrap_or_else(|err| {
        println!("Parse error: {}", err);
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

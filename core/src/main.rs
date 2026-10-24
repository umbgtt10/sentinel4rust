//! `sentinel4rust --roster <file> [--once]`.

use std::process::ExitCode;

use sentinel4rust::cli::{Invocation, USAGE, parse_args, run};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args) {
        Ok(Invocation::Help) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Invocation::Version) => {
            println!("sentinel4rust {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Invocation::Run { roster, once }) => match run(&roster, once) {
            Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
            Err(error) => {
                eprintln!("sentinel4rust: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("sentinel4rust: {error}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

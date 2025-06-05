mod html;

use std::{
    io::{self, Error, Read},
    process::ExitCode,
};

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run() -> Result<(), Error> {
    let mut md = String::new();
    io::stdin().lock().read_to_string(&mut md)?;
    let page = html::make(&md);
    println!("{page}");
    Ok(())
}

use {
    pulldown_cmark::Parser,
    std::{
        io::{self, Error, Read},
        process::ExitCode,
    },
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
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    for event in Parser::new(&input) {
        println!("{event:?}");
    }

    Ok(())
}

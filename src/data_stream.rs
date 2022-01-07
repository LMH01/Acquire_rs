use std::io;

use miette::IntoDiagnostic;
use read_input::prelude::input;

/// Reads a number from stdin
pub fn read_number() -> miette::Result<u8> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer).into_diagnostic()?;
    let number = buffer.trim().parse::<u8>().into_diagnostic()?;
    Ok(number)
}

/// Waits until the user presses enter
pub fn read_enter() {
    input::<char>().default(' ').get();
}

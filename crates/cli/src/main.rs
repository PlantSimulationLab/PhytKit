//! `pk`: plant physiology model fitting from the command line.

use std::io::Write;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (text, code) = phytkit_cli::render(&argv, phytkit_cli::run(&argv));
    // `pk ... | head` closes the pipe early; that's not an error.
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
    std::process::exit(code);
}

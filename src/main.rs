// use inline_colorization::*
use std::process::exit;

fn main() {
    if let Err(error) = nostalgia::run() {
        eprintln!("Error: {error:#}");
        exit(1)
    }
}
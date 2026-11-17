// use inline_colorization::*


fn main() {
    if let Err(error) = nostalgia::run() {
        eprintln!("Error: {:#}", error);
        std::process::exit(1)
    }
}
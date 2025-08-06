mod settings;
mod formater;
mod cli;
mod doc;

use std::env;
fn main() {
    use cli;

    let args: Vec<String> = env::args().collect();
    cli::main(args);
}

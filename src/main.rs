use std::env;
fn main() {
    use sqlformater::cli;

    let args: Vec<String> = env::args().collect();
    cli::main(args);
}

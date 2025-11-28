use std::env;

use build_your_own_utils::my_own_error::MyOwnError;
use install::install_cli;

fn main() -> Result<(), MyOwnError> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|s| &**s).collect::<Vec<&str>>();
    install_cli(&args)
}

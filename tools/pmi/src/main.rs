use std::env;

use build_your_own_utils::my_own_error::MyOwnResult;
use pmi::pmi_cli;

fn main() -> MyOwnResult<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|s| &**s).collect::<Vec<&str>>();
    pmi_cli(&args)
}

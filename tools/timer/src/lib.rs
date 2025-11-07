use std::time::Duration;

use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::MyOwnError;

// My idea!
pub fn timer_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let options = TimerOptions::from_args(args)?;
    if let Some(time) = options.time {
        start_timer(parse_duration(time)?);
    } else {
        start_stopwatch();
    }
    Ok(())
}

fn start_stopwatch() {
    todo!()
}

fn start_timer(time: Duration) {
    todo!()
}

fn parse_duration(time: &str) -> Result<Duration, MyOwnError> {
    todo!()
}

cli_options! {
    struct TimerOptions<'a> {
        #[option()]
        time: Option<&'a str>,
    }
}

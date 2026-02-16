use std::{
    io::{Write, stdout},
    time::{Duration, Instant},
};

use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::MyOwnResult;
use crossterm::{
    cursor::{RestorePosition, SavePosition},
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal,
};

// My idea!
pub fn timer_cli(args: &[&str]) -> MyOwnResult<()> {
    let options = TimerOptions::from_args(args)?;
    if let Some(time) = options.time {
        start_timer(stdout(), parse_duration(time)?)?;
    } else {
        start_stopwatch(stdout())?;
    }
    Ok(())
}

fn start_stopwatch(mut output: impl Write) -> MyOwnResult<()> {
    terminal::enable_raw_mode()?;
    let frame_time = Duration::from_millis(50);
    execute!(output, SavePosition)?;
    let start_time = Instant::now();
    let mut last_checkpoint = start_time;

    loop {
        let now = Instant::now();
        let current = now - start_time;
        execute!(
            output,
            RestorePosition,
            Print(format!(
                "{:0>2}:{:0>2}:{:0>2}",
                current.as_secs() / 60,
                current.as_secs() % 60,
                current.as_millis() % 1000 / 10
            ))
        )?;

        if event::poll(frame_time)? {
            let evt = event::read()?;
            match evt {
                event::Event::Key(key_event) => match key_event {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        ..
                    } if key_event.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyEvent {
                        code: KeyCode::Char(' '), // spacebar
                        ..
                    } => {
                        let duration = now - last_checkpoint;
                        last_checkpoint = now;
                        execute!(
                            output,
                            Print(format!(
                                " ({:0>2}:{:0>2}:{:0>2})\r\n",
                                duration.as_secs() / 60,
                                duration.as_secs() % 60,
                                duration.as_millis() % 1000 / 10
                            )),
                            SavePosition
                        )?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    execute!(output, Print("\r\nEnd!"))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn start_timer(mut output: impl Write, time: Duration) -> MyOwnResult<()> {
    terminal::enable_raw_mode()?;
    let frame_time = Duration::from_millis(50);
    execute!(output, SavePosition)?;
    let target_time = Instant::now() + time;

    loop {
        let now = Instant::now();
        if now > target_time {
            break;
        }
        let left = target_time - now;
        execute!(
            output,
            RestorePosition,
            Print(format!(
                "{}:{:0>2}:{:0>2}",
                left.as_secs() / 60,
                left.as_secs() % 60,
                left.as_millis() % 1000 / 10
            ))
        )?;

        if event::poll(frame_time)? {
            let evt = event::read()?;
            match evt {
                event::Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    execute!(output, Print("\r\nEnd!"))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn parse_duration(time: &str) -> MyOwnResult<Duration> {
    let mut n: u64 = 0;
    let mut measurement = "";
    for (i, byte) in time.bytes().enumerate() {
        if byte.is_ascii_digit() {
            n = n * 10 + byte as u64 - 48;
        } else {
            measurement = &time[i..];
            break;
        }
    }

    match measurement {
        "" | "s" => Ok(Duration::from_secs(n)),
        "m" => Ok(Duration::from_secs(n * 60)),
        "h" => Ok(Duration::from_secs(n * 60 * 60)),
        _ => Err(format!(
            "[{}] is not a valid measure for the amount of time [{}], valid are s, m, h. No measure defaults to seconds.",
            measurement,
            n
        )
        .into()),
    }
}

cli_options! {
    struct TimerOptions<'a> {
        #[option()]
        time: Option<&'a str>,
    }
}

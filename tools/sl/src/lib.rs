use build_your_own_utils::my_own_error::MyOwnError;
use cursive::view::Scrollable;
use cursive::views::{Dialog, SelectView};
use cursive::{Cursive, CursiveExt};
use std::io::{stderr, stdin, stdout, IsTerminal, Read, Write};

// My idea!
pub fn sl_cli(_: &[&str]) -> Result<(), MyOwnError> {
    if stdin().is_terminal() {
        show_err_and_wait_for_exit("Nothing to read from stdin")
    } else {
        sl(stdin(), stdout())
    }
}

fn sl(mut input: impl Read, mut output: impl Write) -> Result<(), MyOwnError> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let lines = buf.lines().collect::<Vec<&str>>();

    let mut siv = Cursive::default();

    if lines.is_empty() {
        show_err_and_wait_for_exit("Nothing to read from stdin")
    }

    let mut select = SelectView::<String>::new();

    for line in &lines {
        select.add_item(line.to_owned(), line.to_string());
    }

    select.set_on_submit(move |s, selected: &str| {
        s.set_user_data(selected.to_string());
        s.quit();
    });

    siv.add_layer(Dialog::around(select.scrollable()).title("Select a line"));

    siv.run();

    if let Some(selected) = siv.take_user_data::<String>() {
        write!(output, "{}", selected)?;
    } else {
        show_err_and_wait_for_exit("Nothing selected")
    }

    Ok(())
}

fn show_err_and_wait_for_exit(message: &str) -> ! {
    write!(stderr(), "{}, use CTRL + C to break the process.", message).ok();
    stderr().flush().ok();
    loop {}
}

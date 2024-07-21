use std::{
    io::{stdin, stdout, BufRead, BufReader, Read, Write},
    num::ParseIntError,
};

use crate::__::{DescribableError, MyOwnError};

// https://codingchallenges.fyi/challenges/challenge-cut

pub fn cut_cli(args: &[&str]) -> Result<(), MyOwnError> {
    cut_cli_impl(args, stdin(), stdout())
}

fn cut_cli_impl(args: &[&str], input: impl Read, output: impl Write) -> Result<(), MyOwnError> {
    let options = CutCliOptions::from_args(args)?;

    if let Some(input_file) = options.input_file {
        let file =
            std::fs::File::open(input_file).inspect_err(|_| eprintln!("no {} file", input_file))?;

        cut(options.options, file, output)
    } else {
        cut(options.options, input, output)
    }
}

fn cut(options: CutOptions, input: impl Read, mut output: impl Write) -> Result<(), MyOwnError> {
    let mut reader = BufReader::new(input);
    let mut buf = Vec::new();

    while reader.read_until(b'\n', &mut buf)? != 0 {
        let line = String::from_utf8(buf)?.trim_end().to_string();
        if let Some(ref fields) = options.fields {
            let all_fields: Vec<&str> = line.split(options.delimiter).collect();

            write!(
                output,
                "{}",
                fields
                    .iter()
                    .map(|f| all_fields.get(f - 1).map(|s| *s).unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(options.delimiter.to_string().as_str())
            )?;
        }

        writeln!(output)?;
        buf = line.into_bytes();
        buf.clear();
    }
    Ok(())
}

struct CutCliOptions<'a> {
    input_file: Option<&'a str>,
    options: CutOptions,
}

struct CutOptions {
    fields: Option<Vec<usize>>,
    delimiter: char,
}

impl<'a> CutCliOptions<'a> {
    fn from_args(args: &[&'a str]) -> Result<Self, MyOwnError> {
        let mut input_file = None;
        let mut fields = None;
        let mut delimiter = '\t';

        let mut args = args.iter();
        loop {
            let arg = args.next();
            let Some(&arg) = arg else {
                break;
            };

            if arg.starts_with("-f") {
                let fields_arg = if arg == "-f" {
                    args.next()
                        .ok_or_else(|| "-f to have numbers after e.g. -f 1 # -f 1,2 # -f \"1 2\"")?
                } else {
                    arg.trim_start_matches("-f")
                };

                fields = Some(
                    fields_arg
                        .split(&[',', ' '])
                        .map(|f| f.trim().parse())
                        .collect::<Result<Vec<usize>, ParseIntError>>()
                        .describe_error("-f to have numbers e.g. -f1 # -f1,2 # -f 1 | {}")?,
                );
                continue;
            }

            if arg.starts_with("-d") {
                let provided_delimiter = if arg == "-d" {
                    args.next()
                        .ok_or_else(|| "delimiter must be a single character e.g. -d ,")?
                } else {
                    arg.trim_start_matches("-d")
                };
                if provided_delimiter.len() != 1 {
                    return Err(MyOwnError::ActualError(
                        "delimiter must be a single character e.g. -d,".into(),
                    ));
                }
                delimiter = provided_delimiter.chars().next().unwrap();
                continue;
            }

            if arg == "-" {
                continue;
            }

            input_file = Some(arg);
        }

        Ok(Self {
            input_file,
            options: CutOptions { fields, delimiter },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_field_all_lines() {
        let input: &[u8] = "1\t2\t3\n4\t5\t6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f1"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1\n4\n".to_string()
        );

        let mut output = Vec::new();
        cut_cli_impl(&["-f2"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "2\n5\n".to_string()
        );
    }

    #[test]
    fn cut_using_provided_delimiter() {
        let input: &[u8] = "1s2s3".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f3", "-ds"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "3\n".to_string()
        );
    }

    #[test]
    fn cut_fields_and_delimiter() {
        let input: &[u8] = "1?2?3\n4?5?6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f1,3", "-d?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );
    }

    #[test]
    fn cut_fields_and_delimiter_specified_with_space() {
        let input: &[u8] = "1?2?3\n4?5?6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f", "1,3", "-d", "?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );

        let mut output = Vec::new();
        cut_cli_impl(&["-f", "1 3", "-d", "?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );
    }
}

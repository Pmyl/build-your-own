use std::io::{stdin, stdout, Read, Write};

mod bits;
mod encoder;
mod decoder;
mod targets;

// https://codingchallenges.fyi/challenges/challenge-huffman

pub fn huffman_cli(args: &[&str]) {
    huffman_cli_impl(args, stdin(), stdout());
}

fn huffman_cli_impl<'a>(args: &[&str], input: impl Read, output: impl Write) {
    let options = HuffmanOptions::from_args(args);
    let targets = targets::HuffmanTargets::new(options.input_file, input, options.output_file, output);

    if let HuffmanMode::Encode = options.mode {
        encoder::encode(targets);
    } else {
        decoder::decode(targets);
    }
}

struct HuffmanOptions<'a> {
    input_file: Option<&'a str>,
    output_file: Option<&'a str>,
    mode: HuffmanMode,
}

impl<'a> HuffmanOptions<'a> {
    fn from_args(args: &[&'a str]) -> Self {
        let mut mode = HuffmanMode::Encode;
        let mut input_file = None;
        let mut output_file = None;

        let mut args = args.iter();
        loop {
            let arg = args.next();
            let Some(&arg) = arg else {
                break;
            };

            if arg == "--decode" {
                mode = HuffmanMode::Decode;
                continue;
            }

            if arg == "--encode" {
                mode = HuffmanMode::Encode;
                continue;
            }

            if arg == "--input" {
                input_file = args.next().map(|s| *s);
                continue;
            }

            if arg == "--output" {
                output_file = args.next().map(|s| *s);
                continue;
            }
        }

        Self {
            input_file,
            output_file,
            mode,
        }
    }
}

enum HuffmanMode {
    Encode,
    Decode,
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn encode_decode_should_return_original_input() {
        let mut input: &[u8] = b"super long string here woooooo";
        let original_length = input.len();
        let mut output = Vec::new();

        huffman_cli_impl(&["--encode"], &mut input, &mut output);

        let mut input: &[u8] = &output;
        let new_length = input.len();
        let mut output = Vec::new();
        huffman_cli_impl(&["--decode"], &mut input, &mut output);

        assert!(original_length < new_length);
        assert_eq!(
            String::from_utf8(output.clone()).expect("to do it"),
            "super long string here woooooo".to_string()
        );
    }

    #[test]
    fn encode_decode_file_should_return_original_input() {
        env::set_var("RUST_BACKTRACE", "1");
        huffman_cli_impl(
            &[
                "--encode",
                "--input",
                "src/huffman/small_test.txt",
                "--output",
                "src/huffman/small_test.huffman",
            ],
            stdin(),
            stdout(),
        );
        huffman_cli_impl(
            &[
                "--decode",
                "--input",
                "src/huffman/small_test.huffman",
                "--output",
                "src/huffman/small_test_result.txt",
            ],
            stdin(),
            stdout(),
        );

        let mut initial_file =
            std::fs::File::open("src/huffman/small_test.txt").expect("file not found");
        let mut result_file =
            std::fs::File::open("src/huffman/small_test_result.txt").expect("file not found");

        let mut initial_content = Vec::new();
        initial_file
            .read_to_end(&mut initial_content)
            .expect("to work");

        let mut result_content = Vec::new();
        result_file
            .read_to_end(&mut result_content)
            .expect("to work");

        let initial = String::from_utf8(initial_content).expect("to do it");
        let result = String::from_utf8(result_content).expect("to do it");

        assert_eq!(result, initial);
    }
}

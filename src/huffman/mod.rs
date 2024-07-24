use std::io::{stdin, stdout, Read, Write};

use build_your_own_macros::cli_options;
use targets::HuffmanInput;

use build_your_own_shared::my_own_error::MyOwnError;

mod bits;
mod decoder;
mod encoder;
mod targets;

// https://codingchallenges.fyi/challenges/challenge-huffman

pub fn huffman_cli(args: &[&str]) -> Result<(), MyOwnError> {
    huffman_cli_impl(args, stdin(), stdout())
}

fn huffman_cli_impl<'a>(
    args: &[&str],
    input: impl Read,
    mut output: impl Write,
) -> Result<(), MyOwnError> {
    let options = HuffmanOptions::from_args(args)?;
    let input = HuffmanInput::new(options.input_file, input);

    if let HuffmanMode::Encode = options.mode {
        encoder::encode(input, &mut output)
    } else {
        decoder::decode(input, &mut output)
    }
}

cli_options! {
    struct HuffmanOptions<'a> {
        #[option()]
        input_file: Option<&'a str>,

        #[option_enum(name = "--decode", variant = HuffmanMode::Decode)]
        #[option_enum(name = "--encode", variant = HuffmanMode::Encode, default = true)]
        mode: HuffmanMode,
    }
}

enum HuffmanMode {
    Encode,
    Decode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_should_return_original_input() {
        let mut input: &[u8] = b"super long string here woooooo";
        let original_length = input.len();
        let mut output = Vec::new();

        huffman_cli_impl(&["--encode"], &mut input, &mut output).expect("to work");

        let mut input: &[u8] = &output;
        let new_length = input.len();
        let mut output = Vec::new();
        huffman_cli_impl(&["--decode"], &mut input, &mut output).expect("to work");

        assert!(original_length < new_length);
        assert_eq!(
            String::from_utf8(output.clone()).expect("to do it"),
            "super long string here woooooo".to_string()
        );
    }

    #[test]
    fn encode_decode_file_should_return_original_input() {
        let mut output = Vec::new();
        huffman_cli_impl(
            &["--encode", "src/huffman/small_test.txt"],
            stdin(),
            &mut output,
        )
        .expect("to work");

        let mut result_content = Vec::new();
        huffman_cli_impl(&["--decode"], output.as_slice(), &mut result_content).expect("to work");

        let mut initial_file =
            std::fs::File::open("src/huffman/small_test.txt").expect("file not found");

        let mut initial_content = Vec::new();
        initial_file
            .read_to_end(&mut initial_content)
            .expect("to work");

        let initial = String::from_utf8(initial_content).expect("to do it");
        let result = String::from_utf8(result_content).expect("to do it");

        assert_eq!(result, initial);
    }
}

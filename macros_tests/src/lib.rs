#[cfg(test)]
mod tests {
    use build_your_own_macros::cli_options;
    cli_options! {
        #[options(
            usage = "app_name some options here",
            examples = &[
                "Example with string: app_name -a -c something",
                "Example without string: app_name -b"
            ]
        )]
        struct TestOptions {
            #[suboptions(name = "flags")]
            struct Flags {
                #[option(name = "-a", descr = "test_bool1_description")]
                test_bool1: bool,
                #[option(name = "-b", descr = "test_bool2_description")]
                test_bool2: bool,
            },

            #[suboptions(name = "options")]
            struct Options {
                #[option(name = "-c", descr = "test_string_description")]
                test_string: String,
                #[option(name = "-d", descr = "test_option_description")]
                test_option: Option<String>,
            },

            #[option(
                name = "-e",
                alt_names = &["--enumber", "-----super-enumber"],
                descr = "test_number_description"
            )]
            test_number: usize,
            #[option(
                name = "-f",
                alt_names = &["--ff"],
                descr = "test_number2_description\nnewline"
            )]
            test_number2: usize,
            #[option(descr = "test_default1_description")]
            test_default1: String,
            #[option(descr = "test_default2_description")]
            test_default2: Option<usize>,
        }
    }

    #[test]
    fn options() {
        let options = TestOptions::from_args(&vec![
            "-a",
            "-c",
            "something",
            "-----super-enumber",
            "10",
            "--ff22",
            "def",
            "32",
        ])
        .unwrap();
        assert_eq!(options.flags.test_bool1, true);
        assert_eq!(options.flags.test_bool2, false);
        assert_eq!(options.options.test_string, "something");
        assert_eq!(options.options.test_option.is_none(), true);
        assert_eq!(options.test_number, 10);
        assert_eq!(options.test_number2, 22);
        assert_eq!(options.test_default1, "def");
        assert_eq!(options.test_default2.unwrap(), 32);

        let mut output: Vec<u8> = vec![];
        options.print_help(&mut output).expect("Write to work");
        let output = String::from_utf8(output).expect("Output to be a string");
        assert_contains(&output, "Flags");
        assert_contains(&output, "Options");
        assert_contains(&output, "-a");
        assert_contains(&output, "test_bool1_description");
        assert_contains(&output, "-b");
        assert_contains(&output, "test_bool2_description");
        assert_contains(&output, "-c");
        assert_contains(&output, "test_string_description");
        assert_contains(&output, "-d");
        assert_contains(&output, "test_option_description");
        assert_contains(&output, "-e");
        assert_contains(&output, "--enumber");
        assert_contains(&output, "-----super-enumber");
        assert_contains(&output, "test_number_description");
        assert_contains(&output, "-f");
        assert_contains(&output, "--ff");
        assert_contains(&output, "test_number2_description");
        assert_contains(&output, "test_default1");
        assert_contains(&output, "test_default1_description");
        assert_contains(&output, "test_default2");
        assert_contains(&output, "test_default2_description");
    }

    fn assert_contains(output: &str, expected: &str) {
        assert!(
            output.contains(expected),
            "Expected [{}] to contain [{}]",
            output,
            expected
        );
    }
}

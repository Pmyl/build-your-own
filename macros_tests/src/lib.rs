#[cfg(test)]
mod tests {
    use build_your_own_macros::cli_options;
    cli_options! {
        struct TestOptions {
            #[option(name = "-a")]
            test_bool1: bool,
            #[option(name = "-b")]
            test_bool2: bool,
            #[option(name = "-c")]
            test_string: String,
            #[option(name = "-d")]
            test_option: Option<String>,
            #[option(name = "-e")]
            test_number: usize,
            #[option()]
            test_default1: String,
            #[option()]
            test_default2: Option<usize>,
        }
    }

    #[test]
    fn options() {
        let options =
            TestOptions::from_args(&vec!["-a", "-c", "something", "-e", "10", "def", "32"])
                .unwrap();
        assert_eq!(options.test_bool1, true);
        assert_eq!(options.test_bool2, false);
        assert_eq!(options.test_string, "something");
        assert_eq!(options.test_option.is_none(), true);
        assert_eq!(options.test_number, 10);
        assert_eq!(options.test_default1, "def");
        assert_eq!(options.test_default2.unwrap(), 32);
    }
}

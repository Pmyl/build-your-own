use std::env;

mod wc;
mod json_checker;
mod tools;
mod huffman;

tools! {
    enum Tool {
        #[tool(
            command = "wc",
            description = "myown wc <options> <file> - myown wc -l file.txt",
            function = wc::wc_cli
        )]
        Wc,
        #[tool(
            command = "json_checker",
            description = "myown json_checker - echo \"{\\\"key\\\": \\\"value\\\"}\" | myown json_checker",
            function = json_checker::json_checker_cli
        )]
        JsonChecker,
        #[tool(
            command = "huffman",
            description = "myown huffman <file> | myown huffman file.txt",
            function = huffman::huffman_cli
        )]
        Zip,
    }
}

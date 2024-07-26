use build_your_own_shared::my_own_error::MyOwnError;
use std::env;

mod cut;
mod huffman;
mod json_checker;
mod redis;
mod tools;
mod wc;
mod xxd;

tools! {
    enum Tool {
        #[tool(
            command = "wc",
            description = "myown wc [-l] [-m] [-w] [-c] [file]",
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
            description = "myown huffman [--encode] [--decode] [file]",
            function = huffman::huffman_cli
        )]
        Huffman,
        #[tool(
            command = "cut",
            description = "myown cut [-f] [-d] [-c] [file]",
            function = cut::cut_cli
        )]
        Cut,
        #[tool(
            command = "redis",
            description = "#WIP# myown redis",
            function = redis::redis_cli
        )]
        Redis,
        #[tool(
            command = "xxd",
            description = "#WIP# myown xxd",
            function = xxd::xxd_cli
        )]
        Xxd,
    }
}

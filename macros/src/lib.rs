use proc_macro::TokenStream;

mod cli_options;

#[proc_macro]
pub fn cli_options(input: TokenStream) -> TokenStream {
    cli_options::cli_options(input)
}

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, LitChar, LitStr, PathArguments, Type,
};

mod new;

#[proc_macro]
pub fn cli_options(input: TokenStream) -> TokenStream {
    new::cli_options(input)
}

#[proc_macro_derive(FromArgs, attributes(option))]
pub fn derive_from_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let data = match input.data {
        Data::Struct(ref data) => data,
        _ => panic!("FromArgs can only be used with structs"),
    };

    let mut option_parsers = Vec::new();
    let mut default_option_parser = None;

    if let Fields::Named(ref fields) = data.fields {
        for field in &fields.named {
            let field_name = &field.ident;
            let mut option_name = None;
            let mut option_delimiter = Vec::new();

            for attr in &field.attrs {
                if attr.path().is_ident("option") {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            option_name = Some(meta.value()?.parse::<LitStr>()?.value());
                            Ok(())
                        } else if meta.path.is_ident("delimiter") {
                            option_delimiter.push(meta.value()?.parse::<LitChar>()?.value());
                            Ok(())
                        } else {
                            Err(meta.error("unsupported attribute"))
                        }
                    })
                    .ok();
                }
            }

            if option_delimiter.is_empty() {
                option_delimiter.push(',');
                option_delimiter.push(' ');
            }

            let field_ty = &field.ty;
            let underlying_type = get_underlying_type(&field_ty);

            let split_multiple_values = if underlying_type.is_multiple {
                quote! {
                    arg_value
                        .split(&[#(#option_delimiter,)*])
                }
            } else {
                quote! {
                    Some(arg_value)
                        .into_iter()
                }
            };

            let parse_value = if underlying_type.needs_parsing() {
                quote! {
                    .map(|arg| arg.parse::<#underlying_type>())
                }
            } else {
                quote! {}
            };

            let collect_parsing = if underlying_type.needs_parsing() {
                quote! {
                    .collect::<Result<Vec<_>, _>>()
                    // TODO: add option to describe parsing error
                    .describe_error("error here")?
                }
            } else {
                quote! {
                    .collect::<Vec<_>>()
                }
            };

            let get_value = if underlying_type.is_multiple {
                quote! {}
            } else if underlying_type.is_option {
                quote! {
                    .into_iter()
                    .nth(0)
                }
            } else {
                quote! {
                    .into_iter()
                    .nth(0)
                    .ok_or_else(|| "mandatory field should be there")?
                }
            };

            let parse_arg = quote! {
                #split_multiple_values
                #parse_value
                #collect_parsing
                #get_value
            };

            if let Some(option_name) = option_name {
                let parse_named_arg = quote! {
                    if arg.starts_with(#option_name) {
                        let arg_value = if *arg == #option_name {
                            args.next()
                                .ok_or_else(|| "not working")?
                        } else {
                            arg.trim_start_matches(#option_name)
                        };

                        options.#field_name = #parse_arg;
                        continue;
                    }
                };

                option_parsers.push(parse_named_arg);
            } else {
                default_option_parser = Some(quote! {
                    let arg_value = arg;
                    options.#field_name = #parse_arg;
                });
            }
        }
    }

    let generics = input.generics;
    let lifetime = generics.lifetimes().next().map(|lt| quote!(#lt));

    let expanded = quote! {
        impl #generics #name #generics {
            pub fn from_args(args: &[&#lifetime str]) -> Result<Self, MyOwnError> {
                let mut options = #name::default();

                let mut args = args.iter();
                while let Some(arg) = args.next() {
                    #(#option_parsers)*
                    #default_option_parser
                }

                Ok(options)
            }
        }
    };

    TokenStream::from(expanded)
}

struct TypeDescription<'a> {
    ident: &'a Ident,
    is_reference: bool,
    is_option: bool,
    is_multiple: bool,
}

impl TypeDescription<'_> {
    fn needs_parsing(&self) -> bool {
        self.ident != "str"
    }
}

impl ToTokens for TypeDescription<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident;
        let reference = if self.is_reference {
            quote! { & }
        } else {
            quote! {}
        };

        tokens.extend(quote! {
            #reference #ident
        });
    }
}

fn get_underlying_type(ty: &Type) -> TypeDescription {
    match &ty {
        syn::Type::Path(type_path) => {
            let type_value = &type_path.path.segments.last().unwrap();
            let type_ident = &type_value.ident;
            if type_ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &type_value.arguments {
                    match args.args.first().unwrap() {
                        syn::GenericArgument::Type(syn::Type::Path(ty)) => TypeDescription {
                            ident: &ty.path.segments.first().unwrap().ident,
                            is_reference: false,
                            is_option: true,
                            is_multiple: false,
                        },
                        syn::GenericArgument::Type(syn::Type::Reference(reference)) => {
                            if let syn::Type::Path(ty) = reference.elem.as_ref() {
                                TypeDescription {
                                    ident: &ty.path.segments.first().unwrap().ident,
                                    is_reference: true,
                                    is_option: true,
                                    is_multiple: false,
                                }
                            } else {
                                panic!("Option reference type must be a simple path type")
                            }
                        }
                        _ => panic!("Option type must be a simple path type"),
                    }
                } else {
                    panic!("Option must have a type")
                }
            } else if type_ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &type_value.arguments {
                    match args.args.first().unwrap() {
                        syn::GenericArgument::Type(syn::Type::Path(ty)) => TypeDescription {
                            ident: &ty.path.segments.first().unwrap().ident,
                            is_reference: false,
                            is_option: false,
                            is_multiple: true,
                        },
                        syn::GenericArgument::Type(syn::Type::Reference(reference)) => {
                            if let syn::Type::Path(ty) = reference.elem.as_ref() {
                                TypeDescription {
                                    ident: &ty.path.segments.first().unwrap().ident,
                                    is_reference: true,
                                    is_option: false,
                                    is_multiple: true,
                                }
                            } else {
                                panic!("Vec reference type must be a simple path type")
                            }
                        }
                        _ => panic!("Vec type must be a simple path type"),
                    }
                } else {
                    panic!("Vec must have a type")
                }
            } else {
                TypeDescription {
                    ident: type_ident,
                    is_reference: false,
                    is_option: false,
                    is_multiple: false,
                }
            }
        }
        syn::Type::Reference(reference) => {
            if let syn::Type::Path(ty) = reference.elem.as_ref() {
                TypeDescription {
                    ident: &ty.path.segments.first().unwrap().ident,
                    is_reference: true,
                    is_option: false,
                    is_multiple: false,
                }
            } else {
                panic!("Option reference type must be a simple path type")
            }
        }
        _ => panic!("Type must be a simple path type or &str"),
    }
}

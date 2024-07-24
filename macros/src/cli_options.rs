use std::iter::Peekable;

use proc_macro2::{token_stream::IntoIter, Group, Ident, Literal, Punct, Span, TokenTree};
use quote::{quote, ToTokens};

pub fn cli_options(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let mut tokens = input.into_iter().peekable();
    let main_struct = match_struct(&mut tokens).expect("expected struct");

    if let Some(_) = tokens.peek() {
        panic!("expected end of input");
    }

    let impl_code = impl_code(&main_struct);
    let structs_code = struct_to_token_stream(main_struct);

    proc_macro::TokenStream::from(quote! {
        #structs_code
        #impl_code
    })
}

fn impl_code(main_struct: &MyOwnStruct) -> proc_macro2::TokenStream {
    let name = &main_struct.name;
    let lifetime = &main_struct.lifetime_token_stream();
    let lifetime_generics = lifetime.as_ref().map(|lifetime| quote! { <#lifetime> });

    let mut update_arg_statements = main_struct
        .structs
        .iter()
        .map(|s| {
            let name = &s.name;
            quote! {
                if options.#name.update_arg(&arg, &mut args)? {
                    continue;
                }
            }
        })
        .collect::<Vec<_>>();

    update_arg_statements.push(quote! {
        if options.update_arg(&arg, &mut args)? {
            continue;
        }
    });

    let from_args_implementations = quote! {
        impl #lifetime_generics #name #lifetime_generics {
            fn from_args(args: &[&#lifetime str]) -> Result<Self, build_your_own_shared::my_own_error::MyOwnError> {
                let mut options = <#name as core::default::Default>::default();

                let mut args = args.iter();
                while let Some(arg) = args.next() {
                    #(#update_arg_statements)*
                    return Err(build_your_own_shared::my_own_error::MyOwnError::ActualError(format!("unknown argument: {}", arg).into()));
                }

                Ok(options)
            }
        }
    };

    let mut update_args_implementations: Vec<proc_macro2::TokenStream> = main_struct
        .structs
        .iter()
        .map(|s| update_arg(&s.strct))
        .collect();

    update_args_implementations.push(update_arg(&main_struct));

    let mut default_implementations: Vec<proc_macro2::TokenStream> = main_struct
        .structs
        .iter()
        .map(|s| default_impl(&s.strct))
        .collect();

    default_implementations.push(default_impl(&main_struct));

    quote! {
        #from_args_implementations
        #(#update_args_implementations)*
        #(#default_implementations)*
    }
}

fn default_impl(strct: &MyOwnStruct) -> proc_macro2::TokenStream {
    let name = &strct.name;
    let lifetime = &strct.lifetime_token_stream();
    let lifetime_generics = lifetime.as_ref().map(|lifetime| quote! { <#lifetime> });

    let default_field_statements = strct
        .fields
        .iter()
        .map(|f| {
            let name = &f.0.name;
            let default =
                &f.1.default
                    .as_ref()
                    .map(|d| d.to_token_stream())
                    .unwrap_or_else(|| quote! { Default::default() });
            quote! {
                #name: #default
            }
        })
        .collect::<Vec<_>>();

    let default_enum_field_statements = strct
        .enums
        .iter()
        .map(|f| {
            let name = &f.0.name;
            let default =
                &f.1.default
                    .as_ref()
                    .map(|d| d.to_token_stream())
                    .unwrap_or_else(|| quote! { Default::default() });
            quote! {
                #name: #default
            }
        })
        .collect::<Vec<_>>();

    let default_struct_statements = strct
        .structs
        .iter()
        .map(|f| {
            let name = &f.name;
            quote! {
                #name: core::default::Default::default()
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #lifetime_generics core::default::Default for #name #lifetime_generics {
            #[inline]
            fn default() -> Self {
                Self {
                    #(#default_field_statements,)*
                    #(#default_enum_field_statements,)*
                    #(#default_struct_statements,)*
                }
            }
        }
    }
}

fn update_arg(strct: &MyOwnStruct) -> proc_macro2::TokenStream {
    let name = &strct.name;
    let lifetime = &strct.lifetime_token_stream();
    let lifetime_generics = lifetime.as_ref().map(|lifetime| quote! { <#lifetime> });

    let mut option_parsers = Vec::new();
    let mut default_option_parser = None;

    strct.fields.iter().for_each(|f| {
        let field_name = &f.0.name;

        match (&f.1.name, f.0.ty.is_bool()) {
            (Some(option_name), false) => {
                let parse_arg = parse_arg(&f);
                let extract_arg_value = extract_arg_value();
                option_parsers.push(quote! {
                    let option_name = #option_name;
                    if arg.starts_with(option_name) {
                        #extract_arg_value
                        self.#field_name = #parse_arg;
                        return Ok(true);
                    }
                })
            }
            (Some(option_name), true) => {
                let parse_arg = if let MyOwnType::Option { .. } = f.0.ty {
                    quote! { Some(true) }
                } else {
                    quote! { true }
                };
                option_parsers.push(quote! {
                    let option_name = #option_name;
                    if arg.starts_with(option_name) {
                        self.#field_name = #parse_arg;
                        return Ok(true);
                    }
                })
            }
            (None, false) => {
                if default_option_parser.is_some() {
                    panic!("multiple fields without option name are not supported");
                }

                let parse_arg = parse_arg(&f);
                default_option_parser = Some(quote! {
                    let arg_value = arg;
                    self.#field_name = #parse_arg;
                    Ok(true)
                });
            }
            (None, true) => panic!("bool fields must have option name"),
        }
    });

    strct.enums.iter().for_each(|f| {
        let field_name = &f.0.name;

        f.1.variants.iter().for_each(|v| {
            let variant_name = &v.name;
            let variant = &v.variant;
            option_parsers.push(quote! {
                if arg == #variant_name {
                    self.#field_name = #variant;
                    return Ok(true);
                }
            });
        });
    });

    let default_option_parser = default_option_parser.unwrap_or_else(|| {
        quote! {
            Ok(false)
        }
    });

    quote! {
        impl #lifetime_generics #name #lifetime_generics {
            fn update_arg(&mut self, arg: &#lifetime str, args: &mut core::slice::Iter<&#lifetime str>) -> Result<bool, build_your_own_shared::my_own_error::MyOwnError> {
                #(#option_parsers)*
                #default_option_parser
            }
        }
    }
}

fn parse_arg(f: &(MyOwnStructComponentField, MyOwnFieldAttribute)) -> proc_macro2::TokenStream {
    let split_multiple_values = if let MyOwnType::Vec { .. } = f.0.ty {
        let delimiters = f.1.delimiters();
        quote! {
            arg_value
                .split(&[#(#delimiters,)*])
        }
    } else {
        quote! {
            Some(arg_value)
                .into_iter()
        }
    };

    let parse_value = if f.0.ty.needs_parsing() {
        let ty = f.0.ty.to_parse();
        quote! {
            .map(|arg| arg.parse::<#ty>())
            .collect::<Result<Vec<_>, _>>()
            // TODO: add option to describe parsing error
            .map_err(|e| build_your_own_shared::my_own_error::MyOwnError::ActualErrorWithDescription(e.into(), format!("error parsing {}", arg).into()))?
        }
    } else {
        quote! {
            .collect::<Vec<_>>()
        }
    };

    let get_value = match f.0.ty {
        MyOwnType::Vec { .. } => quote! {},
        MyOwnType::Option { .. } => {
            quote! {
                .into_iter()
                .nth(0)
            }
        }
        _ => {
            quote! {
                .into_iter()
                .nth(0)
                .ok_or_else(|| "mandatory field should be there")?
            }
        }
    };

    quote! {
        #split_multiple_values
        #parse_value
        #get_value
    }
}

fn extract_arg_value() -> proc_macro2::TokenStream {
    quote! {
        let arg_value = if arg == option_name {
            args.next()
                .ok_or_else(|| "not working")?
        } else {
            arg.trim_start_matches(option_name)
        };
    }
}

fn struct_to_token_stream(main_struct: MyOwnStruct) -> proc_macro2::TokenStream {
    let mut structs = vec![main_struct];
    let mut structs_code = Vec::new();

    loop {
        if structs.is_empty() {
            break;
        }

        let strct = structs.pop().unwrap();
        let fields = strct.fields.iter().map(|f| {
            let name = &f.0.name;
            let ty = f.0.ty.to_token_stream();
            quote! {
                #name: #ty
            }
        });
        let enum_fields = strct.enums.iter().map(|f| {
            let name = &f.0.name;
            let ty = f.0.ty.to_token_stream();
            quote! {
                #name: #ty
            }
        });
        let struct_fields = strct.structs.iter().map(|f| {
            let name = &f.name;
            let ty = &f.strct.name;
            quote! {
                #name: #ty
            }
        });
        let name = &strct.name;
        let lifetime = &strct.lifetime_token_stream();
        let lifetime_generics = lifetime.as_ref().map(|lifetime| quote! { <#lifetime> });
        structs_code.push(quote! {
            struct #name #lifetime_generics {
                #(#fields,)*
                #(#enum_fields,)*
                #(#struct_fields,)*
            }
        });
        strct
            .structs
            .into_iter()
            .for_each(|s| structs.push(s.strct));
    }

    quote! {
        #(#structs_code)*
    }
}

fn match_struct(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnStruct> {
    let struct_ident = tokens.next();
    let TokenTree::Ident(ident) = struct_ident.expect("to start with `struct`") else {
        panic!("expected `struct`");
    };
    if ident != "struct" {
        panic!("expected `struct`");
    }

    let struct_name_token = tokens.next();
    let TokenTree::Ident(struct_name_ident) =
        struct_name_token.expect("to start with `struct StructName`")
    else {
        panic!("expected struct name");
    };

    let lifetime = if let Some(_) = peek_punct_value(tokens, '<') {
        let open_lifetime = match_punct_value(tokens, '<');
        let apostrophe = match_punct_value(tokens, '\'');
        let ident = match_ident(tokens).expect("should have lifetime");
        let close = match_punct_value(tokens, '>');
        Some((open_lifetime, apostrophe, ident, close))
    } else {
        None
    };

    let group = match_struct_group(tokens);
    let mut group_tokens = group.stream().into_iter().peekable();
    let mut structs = Vec::new();
    let mut fields = Vec::new();
    let mut enums = Vec::new();

    loop {
        let attribute = match_attribute(&mut group_tokens).expect("to find attribute");

        match attribute {
            MyOwnAttribute::Field(field_attribute) => {
                let field = match_struct_field(&mut group_tokens).expect("field after attribute");
                fields.push((field, field_attribute));
            }
            MyOwnAttribute::Struct(suboptions_name) => {
                structs.push(MyOwnStructComponentStruct {
                    name: Ident::new(suboptions_name.as_str(), Span::call_site()),
                    strct: match_struct(&mut group_tokens)
                        .expect("struct after `suboptions` attribute"),
                });
            }
            MyOwnAttribute::EnumField(enum_attribute) => {
                let field =
                    match_struct_field(&mut group_tokens).expect("field after enum attributes");
                enums.push((field, enum_attribute));
            }
        }

        maybe_match_punct_value(&mut group_tokens, ',');

        if let None = group_tokens.peek() {
            break;
        }
    }

    Some(MyOwnStruct {
        name: struct_name_ident,
        lifetime,
        fields,
        enums,
        structs,
    })
}

fn match_struct_field(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnStructComponentField> {
    let field_name = match_ident(tokens)?;
    match_punct_value(tokens, ':');
    let field_type = match_type(tokens).expect("type of field to be there");

    Some(MyOwnStructComponentField {
        name: field_name,
        ty: field_type,
    })
}

fn match_type(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnType> {
    let token = tokens.next()?;

    match token {
        TokenTree::Ident(ident) => {
            if ident == "Option" {
                Some(MyOwnType::Option {
                    ident,
                    open: match_punct_value(tokens, '<'),
                    ty: Box::new(match_type(tokens).expect("inner type of `Option`")),
                    close: match_punct_value(tokens, '>'),
                })
            } else if ident == "Vec" {
                Some(MyOwnType::Vec {
                    ident,
                    open: match_punct_value(tokens, '<'),
                    ty: Box::new(match_type(tokens).expect("inner type of `Vec`")),
                    close: match_punct_value(tokens, '>'),
                })
            } else {
                Some(MyOwnType::Name {
                    ident,
                    reference: None,
                    lifetime: None,
                })
            }
        }
        TokenTree::Punct(punct) => {
            if punct.as_char() == '&' {
                if let Some(TokenTree::Punct(lifetime_apostrophe)) = tokens.peek() {
                    let lifetime = if lifetime_apostrophe.as_char() == '\'' {
                        Some((
                            match_punct_value(tokens, '\''),
                            match_ident(tokens).expect("lifetime name"),
                        ))
                    } else {
                        None
                    };
                    let type_name = match_ident(tokens).expect("type name");
                    return Some(MyOwnType::Name {
                        ident: type_name,
                        reference: Some(punct),
                        lifetime,
                    });
                }
            }
            panic!("unexpected punctuation when matching type");
        }
        TokenTree::Literal(_) => {
            panic!("unexpected literal when matching type");
        }
        TokenTree::Group(_) => {
            panic!("unexpected group when matching type");
        }
    }
}

fn match_struct_group(tokens: &mut Peekable<IntoIter>) -> Group {
    let token = tokens.next().expect("a token to be there");

    match token {
        TokenTree::Group(group) => {
            if group.delimiter() != proc_macro2::Delimiter::Brace {
                panic!("expected `{{` to open a struct");
            }

            group
        }
        _ => panic!("expected `{{`"),
    }
}

fn match_attribute(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnAttribute> {
    if let None = peek_punct_value(tokens, '#') {
        return None;
    }
    match_punct_value(tokens, '#');

    let group =
        match_group(tokens, proc_macro2::Delimiter::Bracket).expect("expected `[` after `#`");

    let mut attributes_tokens = group.stream().into_iter().peekable();
    let attribute_name =
        match_ident(&mut attributes_tokens).expect("expected `option` or `suboptions` attribute");

    if attribute_name == "option" {
        let mut group_tokens =
            match_group(&mut attributes_tokens, proc_macro2::Delimiter::Parenthesis)
                .expect("expected `(` after `option`")
                .stream()
                .into_iter()
                .peekable();

        if let Some(_) = attributes_tokens.next() {
            panic!("unexpected attribute, should only have one attribute per #[..] block");
        };

        let mut attribute = MyOwnFieldAttribute::default();

        loop {
            let field = match_attribute_field(&mut group_tokens);

            match field {
                Some(field) => match field.name.as_ref() {
                    "name" => attribute.name = Some(field.value_as_string()),
                    "delimiters" => attribute.delimiters = Some(field.value_as_char_vec()),
                    "default" => attribute.default = Some(field.value),
                    unhandled => panic!("unhandled attribute field `{}`", unhandled),
                },
                None => break,
            }

            if let None = peek_punct_value(&mut group_tokens, ',') {
                break;
            }
            match_punct_value(&mut group_tokens, ',');
        }

        if let Some(_) = group_tokens.peek() {
            panic!("unexpected token in attribute, all attributes should be separated by `,`");
        }

        // TODO: this loop is useless because the next attribute will find the next and so on until the end
        loop {
            let Some(next_attribute) = match_attribute(tokens) else {
                break;
            };

            let MyOwnAttribute::Field(next_attribute) = next_attribute else {
                panic!("expected option attribute after a previous option attribute");
            };

            attribute.merge(next_attribute);
        }

        Some(MyOwnAttribute::Field(attribute))
    } else if attribute_name == "suboptions" {
        let mut group_tokens =
            match_group(&mut attributes_tokens, proc_macro2::Delimiter::Parenthesis)
                .expect("expected `(` after `option`")
                .stream()
                .into_iter()
                .peekable();

        if let Some(_) = attributes_tokens.next() {
            panic!("unexpected attribute, should only have one attribute per #[..] block");
        };

        let field = match_attribute_field(&mut group_tokens);

        let suboptions_name = match field {
            Some(field) => match field.name.as_ref() {
                "name" => field.value_as_string(),
                unhandled => panic!("unhandled attribute field `{}`", unhandled),
            },
            None => panic!("expected `name` field in `suboptions` attribute"),
        };

        if let Some(_) = group_tokens.peek() {
            panic!("unexpected token in attribute, `suboptions` should only have `name` field");
        }

        Some(MyOwnAttribute::Struct(suboptions_name))
    } else if attribute_name == "option_enum" {
        let mut group_tokens =
            match_group(&mut attributes_tokens, proc_macro2::Delimiter::Parenthesis)
                .expect("expected `(` after `option_enum`")
                .stream()
                .into_iter()
                .peekable();

        if let Some(_) = attributes_tokens.next() {
            panic!("unexpected attribute, should only have one attribute per #[..] block");
        };

        let mut attribute = MyOwnEnumFieldAttribute::default();
        let mut variant = MyOwnEnumFieldVariantAttribute::default();

        loop {
            let field = match_attribute_field(&mut group_tokens);

            match field {
                Some(field) => match field.name.as_ref() {
                    "name" => variant.name = Some(field.value_as_string()),
                    "variant" => variant.variant = Some(field.value),
                    "default" => {
                        if let Some(_) = attribute.default {
                            panic!("`default` variant can only be set once");
                        }

                        if variant.variant.is_none() {
                            panic!("`default` should be set after `variant` field");
                        }

                        attribute.default = variant.variant.clone()
                    }
                    unhandled => panic!("unhandled attribute field `{}`", unhandled),
                },
                None => break,
            }

            if let None = peek_punct_value(&mut group_tokens, ',') {
                break;
            }
            match_punct_value(&mut group_tokens, ',');
        }

        attribute.variants.push(variant);

        if let Some(_) = group_tokens.peek() {
            panic!("unexpected token in attribute, all attributes should be separated by `,`");
        }

        // TODO: this loop is useless because the next attribute will find the next and so on until the end
        loop {
            let Some(next_attribute) = match_attribute(tokens) else {
                break;
            };

            let MyOwnAttribute::EnumField(next_attribute) = next_attribute else {
                panic!("expected option_enum attribute after a previous option_enum attribute");
            };

            attribute.merge(next_attribute);
        }

        Some(MyOwnAttribute::EnumField(attribute))
    } else {
        panic!("unhandled attribute `{}`", attribute_name);
    }
}

fn match_attribute_field(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnAttributeField> {
    let field_name = match_ident(tokens)?;
    match_punct_value(tokens, '=');
    let field_value = match_value(tokens).expect("value of field to be there");

    Some(MyOwnAttributeField {
        name: field_name.to_string(),
        value: field_value,
    })
}

fn match_value(tokens: &mut Peekable<IntoIter>) -> Option<MyOwnValue> {
    let token = tokens.next()?;

    match token {
        TokenTree::Literal(literal) => {
            let literal = literal.to_string();
            if literal.starts_with('\'') {
                Some(MyOwnValue::Char(
                    literal
                        .to_string()
                        .strip_prefix("\'")
                        .expect("expected char to contain \' at the beginning")
                        .strip_suffix("\'")
                        .expect("expected char to contain \' at the end")
                        .replace("\\t", "\t")
                        .replace("\\n", "\n")
                        .replace("\\r", "\r")
                        .parse()
                        .expect("expected char to be parseable"),
                ))
            } else if literal.starts_with('"') {
                Some(MyOwnValue::String(
                    literal
                        .to_string()
                        .strip_prefix("\"")
                        .expect("expected string to contain \" at the beginning")
                        .strip_suffix("\"")
                        .expect("expected string to contain \" at the end")
                        .to_string(),
                ))
            } else if literal == "true" {
                Some(MyOwnValue::Bool(true))
            } else if literal == "false" {
                Some(MyOwnValue::Bool(false))
            } else {
                panic!("unexpected literal value {}", literal);
            }
        }
        TokenTree::Punct(punct) => {
            if punct.as_char() == '&' {
                let mut slice_group = match_group(tokens, proc_macro2::Delimiter::Bracket)
                    .expect("expected `[]` group after `&`")
                    .stream()
                    .into_iter()
                    .peekable();
                let mut values = Vec::<char>::new();
                let literal =
                    match_literal(&mut slice_group).expect("expected a literal after `&[`");
                values.push(
                    literal
                        .to_string()
                        .strip_prefix("'")
                        .unwrap()
                        .strip_suffix("'")
                        .unwrap()
                        .parse()
                        .expect("expected a char literal"),
                );
                loop {
                    if let None = maybe_match_punct_value(&mut slice_group, ',') {
                        break;
                    }
                    let literal = match_literal(&mut slice_group)
                        .expect("expected a literal after `,` in `&[]`");
                    values.push(
                        literal
                            .to_string()
                            .strip_prefix("'")
                            .unwrap()
                            .strip_suffix("'")
                            .unwrap()
                            .parse()
                            .expect("expected a char literal"),
                    );
                }
                if let Some(_) = slice_group.next() {
                    panic!("unexpected token in `&[]` group");
                }

                Some(MyOwnValue::CharVec(values))
            } else {
                panic!("expected `&` punct only");
            }
        }
        TokenTree::Ident(ident) => {
            let mut path = vec![ident];
            while let Some(_) = peek_punct_value(tokens, ':') {
                match_punct_value(tokens, ':');
                match_punct_value(tokens, ':');
                let ident = match_ident(tokens).expect("expected ident after `::`");
                path.push(ident);
            }
            Some(MyOwnValue::Path(path))
        }
        tt => panic!("unexpected value {:?}", tt),
    }
}

fn match_literal(tokens: &mut Peekable<IntoIter>) -> Option<Literal> {
    let token = tokens.next()?;

    match token {
        TokenTree::Literal(literal) => Some(literal),
        tt => panic!("expected a literal but was {:?}", tt),
    }
}

fn maybe_match_punct_value(tokens: &mut Peekable<IntoIter>, expected_punct: char) -> Option<Punct> {
    let token = tokens.next()?;

    match token {
        TokenTree::Punct(punct) => {
            if punct.as_char() != expected_punct {
                panic!(
                    "expected punct `{}` but found punct `{}`",
                    expected_punct,
                    punct.as_char()
                );
            }

            Some(punct)
        }
        tt => panic!("expected punct `{}` but was `{:?}`", expected_punct, tt),
    }
}

fn peek_punct_value(tokens: &mut Peekable<IntoIter>, expected_punct: char) -> Option<&Punct> {
    let token = tokens.peek()?;

    match token {
        TokenTree::Punct(punct) => {
            if punct.as_char() != expected_punct {
                None
            } else {
                Some(&punct)
            }
        }
        _ => None,
    }
}

fn match_punct_value(tokens: &mut Peekable<IntoIter>, expected_punct: char) -> Punct {
    let Some(punct) = maybe_match_punct_value(tokens, expected_punct) else {
        panic!("expected `{}` but found no tokens", expected_punct);
    };

    punct
}

fn match_group(
    tokens: &mut Peekable<IntoIter>,
    expected_delimiter: proc_macro2::Delimiter,
) -> Option<Group> {
    let token = tokens.next()?;

    match token {
        TokenTree::Group(group) => {
            if group.delimiter() != expected_delimiter {
                panic!("expected `{:?}`", expected_delimiter);
            }

            Some(group)
        }
        _ => panic!("expected `{:?}`", expected_delimiter),
    }
}

fn match_ident(tokens: &mut Peekable<IntoIter>) -> Option<Ident> {
    let token = tokens.next()?;

    match token {
        TokenTree::Ident(ident) => Some(ident),
        tt => panic!("expected an identifier but fount {:?}", tt),
    }
}

#[derive(Debug, Default)]
struct MyOwnEnumFieldAttribute {
    variants: Vec<MyOwnEnumFieldVariantAttribute>,
    default: Option<MyOwnValue>,
}

impl MyOwnEnumFieldAttribute {
    fn merge(&mut self, attribute: MyOwnEnumFieldAttribute) {
        if let Some(same) = self
            .variants
            .iter()
            .find(|v| attribute.variants.iter().any(|av| v.name == av.name))
        {
            panic!("Name already specified {:?}", same.name);
        }

        match (&self.default, &attribute.default) {
            (Some(_), Some(_)) => panic!("Default already specified"),
            (None, Some(_)) => self.default = attribute.default,
            _ => {}
        }

        self.variants.extend(attribute.variants);
    }
}

#[derive(Debug, Default)]
struct MyOwnEnumFieldVariantAttribute {
    name: Option<String>,
    // TODO: this should just be the Path value?
    variant: Option<MyOwnValue>,
}

#[derive(Debug, Default)]
struct MyOwnFieldAttribute {
    name: Option<String>,
    delimiters: Option<Vec<char>>,
    default: Option<MyOwnValue>,
}

impl MyOwnFieldAttribute {
    fn delimiters(&self) -> Vec<char> {
        self.delimiters
            .as_ref()
            .cloned()
            .unwrap_or_else(|| vec![','])
    }

    fn merge(&mut self, attribute: MyOwnFieldAttribute) {
        if let Some(name) = attribute.name {
            if let Some(self_name) = &self.name {
                panic!("Name already specified to be {}", self_name);
            }

            self.name = Some(name);
        }

        if let Some(delimiters) = attribute.delimiters {
            if let Some(self_delimiters) = &self.delimiters {
                panic!("Delimiters already specified to be {:?}", self_delimiters);
            }

            self.delimiters = Some(delimiters);
        }

        if let Some(default) = attribute.default {
            if let Some(self_default) = &self.default {
                panic!("Default already specified to be {:?}", self_default);
            }

            self.default = Some(default);
        }
    }
}

#[derive(Debug)]
enum MyOwnAttribute {
    Field(MyOwnFieldAttribute),
    EnumField(MyOwnEnumFieldAttribute),
    Struct(String),
}

#[derive(Debug)]
struct MyOwnAttributeField {
    name: String,
    value: MyOwnValue,
}

impl MyOwnAttributeField {
    fn value_as_string(self) -> String {
        if let MyOwnValue::String(value) = self.value {
            return value;
        }

        panic!(
            "expected value of attribute field {} to be a string",
            self.name
        );
    }

    fn value_as_char_vec(self) -> Vec<char> {
        if let MyOwnValue::CharVec(value) = self.value {
            return value;
        }

        panic!(
            "expected value of attribute field {} to be a char vector",
            self.name
        );
    }
}

#[derive(Debug, Clone)]
enum MyOwnValue {
    Char(char),
    String(String),
    Bool(bool),
    CharVec(Vec<char>),
    Path(Vec<Ident>),
}

impl ToTokens for MyOwnValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MyOwnValue::Char(value) => {
                tokens.extend(quote! {
                    #value
                });
            }
            MyOwnValue::String(value) => {
                tokens.extend(quote! {
                    #value
                });
            }
            MyOwnValue::CharVec(values) => {
                tokens.extend(quote! {
                    vec![#(#values),*]
                });
            }
            MyOwnValue::Bool(value) => {
                tokens.extend(quote! {
                    #value
                });
            }
            MyOwnValue::Path(values) => {
                tokens.extend(quote! {
                    #(#values)::*
                });
            }
        }
    }
}

#[derive(Debug)]
enum MyOwnType {
    Name {
        ident: Ident,
        reference: Option<Punct>,
        lifetime: Option<(Punct, Ident)>,
    },
    Option {
        ident: Ident,
        open: Punct,
        ty: Box<MyOwnType>,
        close: Punct,
    },
    Vec {
        ident: Ident,
        open: Punct,
        ty: Box<MyOwnType>,
        close: Punct,
    },
}

impl MyOwnType {
    fn needs_parsing(&self) -> bool {
        match self {
            MyOwnType::Name {
                reference: Some(_),
                ident,
                ..
            } => ident != "str",
            MyOwnType::Option { ty, .. } => ty.needs_parsing(),
            MyOwnType::Vec { ty, .. } => ty.needs_parsing(),
            _ => true,
        }
    }

    fn is_bool(&self) -> bool {
        match self {
            MyOwnType::Name { ident, .. } => ident == "bool",
            MyOwnType::Option { ty, .. } => ty.is_bool(),
            _ => false,
        }
    }

    fn to_parse(&self) -> proc_macro2::TokenStream {
        match self {
            MyOwnType::Name {
                ident,
                reference,
                lifetime,
            } => {
                let lifetime_apostrophe = lifetime.as_ref().map(|(punct, _)| punct);
                let lifetime_ident = lifetime.as_ref().map(|(_, ident)| ident);
                quote! {
                    #reference #lifetime_apostrophe #lifetime_ident #ident
                }
            }
            MyOwnType::Option { ty, .. } | MyOwnType::Vec { ty, .. } => {
                let ty = ty.to_parse();
                quote! {
                    #ty
                }
            }
        }
    }

    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            MyOwnType::Name {
                ident,
                reference,
                lifetime,
            } => {
                let lifetime_apostrophe = lifetime.as_ref().map(|(punct, _)| punct);
                let lifetime_ident = lifetime.as_ref().map(|(_, ident)| ident);
                quote! {
                    #reference #lifetime_apostrophe #lifetime_ident #ident
                }
            }
            MyOwnType::Option {
                ident,
                open,
                ty,
                close,
            } => {
                let ty = ty.to_token_stream();
                quote! {
                    #ident #open #ty #close
                }
            }
            MyOwnType::Vec {
                ident,
                open,
                ty,
                close,
            } => {
                let ty = ty.to_token_stream();
                quote! {
                    #ident #open #ty #close
                }
            }
        }
    }
}

#[derive(Debug)]
struct MyOwnStructComponentField {
    name: Ident,
    ty: MyOwnType,
}

#[derive(Debug)]
struct MyOwnStructComponentStruct {
    name: Ident,
    strct: MyOwnStruct,
}

#[derive(Debug)]
struct MyOwnStruct {
    name: Ident,
    lifetime: Option<(Punct, Punct, Ident, Punct)>,
    fields: Vec<(MyOwnStructComponentField, MyOwnFieldAttribute)>,
    enums: Vec<(MyOwnStructComponentField, MyOwnEnumFieldAttribute)>,
    structs: Vec<MyOwnStructComponentStruct>,
}

impl MyOwnStruct {
    fn lifetime_token_stream(&self) -> Option<proc_macro2::TokenStream> {
        if let Some((_, ap, ident, _)) = &self.lifetime {
            Some(quote! {
                #ap #ident
            })
        } else {
            None
        }
    }
}

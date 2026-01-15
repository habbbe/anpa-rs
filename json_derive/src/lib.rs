use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Lit, Meta, Type, parse_macro_input};

/// Derive macro to automatically generate a JSON deserializer using
/// macros json_parser_gen, json_parser_gen_exact
#[proc_macro_derive(Deserialize, attributes(anpa))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let unescaped_strings = uses_unescaped_string(&input.attrs);

    #[allow(unused)]
    let (mut impl_generics, mut ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetimes = input.generics.lifetimes().next();

    let mut generics = input.generics.clone();
    let lifetime: syn::LifetimeParam = syn::parse_quote!('anpa_lifetime);

    if lifetimes.is_none() {
        generics.params.insert(0, syn::GenericParam::Lifetime(lifetime.clone()));
        (impl_generics, _, _) = generics.split_for_impl();
    } else {
        generics.params[0] = syn::GenericParam::Lifetime(lifetime.clone());
        (impl_generics, ty_generics, _) = generics.split_for_impl();
    }

    let lifetime_macro = lifetimes
        .map(|_| quote! { #lifetime, })
        .unwrap_or_else(|| quote! {});

    let Data::Struct(data) = &input.data else {
        panic!("Deserialize can only be derived for structs");
    };

    let Fields::Named(fields) = &data.fields else {
        panic!("Deserialize can only be derived for structs with named fields");
    };

    let (field_entries, exact_field_entries): (Vec<_>, Vec<_>) = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let mut field_type = &field.ty;
        let unescaped_string = unescaped_strings || uses_unescaped_string(&field.attrs);
        let json_name = parse_field_name(&field.attrs, field_name);

        let mut optional = false;

        if let Some(inner_type) = extract_inner_if_option(field_type) {
            field_type = inner_type;
            optional = true;
        }

        let new_lifetime_str: Type = syn::parse_quote!(&#lifetime str);

        if let Type::Reference(type_ref) = field_type {
            if let Type::Path(path) = &*type_ref.elem {
                let segment = path.path.segments.last().unwrap();
                let type_name = segment.ident.to_string();

                if type_name == "str" {
                    field_type = &new_lifetime_str;
                }
            }
        }

        let parser = generate_parser_for_type(field_type, false, unescaped_string);
        let parser_exact = generate_parser_for_type(field_type, true, unescaped_string);

        let parser_exact = if optional {
            quote! { ::anpa::json::option_parser(#parser_exact) }
        } else {
            parser_exact
        };

        (
            quote! { (#json_name, #field_name, #field_type, #parser, optional: #optional) },
            quote! { (#json_name, #parser_exact) }
        )
    }).unzip();

    let field_name_list: Vec<_> = fields.named.iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();

    quote! {
        impl #impl_generics ::anpa::json::JsonDeserializable<#lifetime, #name #ty_generics> for #name #ty_generics #where_clause {
            #[inline]
            fn json_parser() -> impl ::anpa::json::JsonParser<#lifetime, #name #ty_generics> {
                ::anpa::json_parser_gen!(#lifetime_macro #name,
                    #(#field_entries),*
                )
            }

            #[inline]
            fn json_parser_exact() -> impl ::anpa::json::JsonParser<#lifetime, #name #ty_generics> {
                ::anpa::json_parser_gen_exact!(|#(#field_name_list),*| #name { #(#field_name_list),* },
                    #(#exact_field_entries),*
                )
            }
        }
    }.into()
}

fn for_nested_metas<T>(attrs: &[Attribute], default: T, mut f: impl FnMut(&Meta) -> Option<T>) -> T {
    for attr in attrs {
        if !attr.path().is_ident("anpa") { continue; }
        let Meta::List(list) = &attr.meta else { continue; };

        let Ok(nested_metas) = list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        ) else {
            continue;
        };

        for nested in nested_metas {
            match f(&nested) {
                Some(v) => return v,
                None => {}
            }
        }
    }

    default
}

fn parse_field_name(attrs: &[Attribute], field_name: &syn::Ident) -> String {
    for_nested_metas(attrs, field_name.to_string(), |meta| {
        let Meta::NameValue(nv) = meta else { return None };
        if !nv.path.is_ident("rename") { return None }

        let syn::Expr::Lit(expr_lit) = &nv.value else { return None };
        let Lit::Str(lit_str) = &expr_lit.lit else { return None };

        Some(lit_str.value())
    })
}

fn uses_unescaped_string(attrs: &[Attribute]) -> bool {
    for_nested_metas(attrs, false, |meta| {
        let Meta::Path(path) = meta else { return None };
        path.is_ident("unescaped_string").then_some(true)
    })
}

fn extract_inner_if_option(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else { return None; };
    let Some(segment) = path.path.segments.last() else { return None; };
    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
        return None;
    };

    Some(inner_ty)
}

fn generate_parser_for_type(ty: &Type, exact: bool, unescaped_string: bool) -> proc_macro2::TokenStream {
    let self_parser = if exact {
        quote! { #ty::json_parser_exact() }
    } else {
        quote! { #ty::json_parser() }
    };

    match ty {
        Type::Reference(type_ref) => {
            if let Type::Path(path) = &*type_ref.elem {
                if path.path.segments.last().unwrap().ident == "str" {
                    return quote! { ::anpa::json::string_parser() };
                }
            }
            self_parser
        }
        Type::Path(path) => {
            let segment = path.path.segments.last().unwrap();
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "String" => if unescaped_string { quote! { ::anpa::json::string_parser() } }
                            else { quote! { ::anpa::json::escaped_string_parser() } },
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => quote! { ::anpa::number::integer() },
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => quote! { ::anpa::number::integer_signed() },
                "f32" | "f64" => quote! { ::anpa::number::float() },
                "bool" => quote! { ::anpa::json::bool_parse() },
                "Vec" => {
                    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                        panic!("Vec type must have generic arguments");
                    };
                    let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
                        panic!("Vec must have a type parameter");
                    };
                    let inner_parser = generate_parser_for_type(inner_ty, exact, unescaped_string);
                    quote! { ::anpa::json::vec_parser(#inner_parser) }
                },
                "Box" => {
                    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                        panic!("Box type must have generic arguments");
                    };
                    let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else {
                        panic!("Box must have a type parameter");
                    };
                    let inner_parser = generate_parser_for_type(inner_ty, exact, unescaped_string);

                    quote! { ::anpa::combinators::map(#inner_parser, Box::new) }
                },
                _ => self_parser
            }
        },
        // For non-path types, assume they implement JsonDeserializable
        _ => self_parser
    }
}

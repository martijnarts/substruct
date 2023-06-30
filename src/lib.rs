use core::panic;

use convert_case::{Case, Casing};
use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, DeriveInput, ItemFn};

trait SubstructRoot {}

fn parse_substruct_root_macro(input: DeriveInput) -> TokenStream2 {
    let data = match input.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Only structs are supported"),
    };

    let struct_ident = input.ident;

    let mut impls: Vec<TokenStream2> = vec![];
    for field in data.fields {
        let ident = field.ident.unwrap();

        let method_name = syn::Ident::new(&ident.to_string().to_case(Case::Snake), ident.span());
        let trait_name = syn::Ident::new(
            &ident.to_string().as_str().to_case(Case::Pascal),
            ident.span(),
        );
        let ty = field.ty;

        impls.push(quote! {
            trait #trait_name {
                fn #method_name(&self) -> &#ty;
            }
            impl #trait_name for #struct_ident {
                fn #method_name(&self) -> &#ty {
                    &self.#ident
                }
            }
        });
    }

    quote! {
        #(#impls)*
    }
}

#[proc_macro_derive(SubstructRoot)]
pub fn substruct_root(orig_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(orig_input as DeriveInput);

    TokenStream::from(parse_substruct_root_macro(input))
}

#[derive(FromMeta)]
#[darling()]
struct SubstructUse {
    #[allow(dead_code)]
    root: syn::ExprPath,
    fields: darling::util::PathList,
}

fn parse_substruct_use(args: TokenStream2, item_fn: ItemFn) -> TokenStream2 {
    let args = match NestedMeta::parse_meta_list(args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream2::from(Error::from(e).write_errors());
        }
    };

    let attr = match SubstructUse::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors();
        }
    };

    let fn_name = item_fn.sig.ident.clone();
    let fn_trait_name = syn::Ident::new(
        &format!("{}Input", fn_name.to_string().to_case(Case::Pascal)),
        fn_name.span(),
    );

    let field_impls = attr.fields.iter().map(|attr| {
        let attr = attr.segments.first().unwrap();
        let attr = syn::Ident::new(
            &attr.ident.to_string().to_case(Case::Pascal),
            attr.ident.span(),
        );

        quote! {
            #attr
        }
    });

    let cloned_impls = field_impls.clone();

    let fn_ident = item_fn.sig.ident.clone();
    let fn_return = item_fn.sig.output.clone();
    let fn_body = item_fn.block;

    quote! {
        trait #fn_trait_name: #(#field_impls)+* {}
        impl<T: #(#cloned_impls)+*> #fn_trait_name for T {}

        fn #fn_ident(query: impl #fn_trait_name) #fn_return
            #fn_body
    }
}

#[proc_macro_attribute]
pub fn substruct_use(attr: TokenStream, fn_input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(fn_input as ItemFn);
    TokenStream::from(parse_substruct_use(attr.into(), input))
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{parse_substruct_root_macro, parse_substruct_use};

    #[test]
    fn test_simple_use() {
        let tokens: syn::File = syn::parse2(parse_substruct_root_macro(parse_quote! {
            #[derive(SubstructRoot)]
            struct Query {
                name: String,
            }
        }))
        .unwrap();

        assert_eq!(tokens.items.len(), 2);

        tokens.items.iter().for_each(|item| {
            if let syn::Item::Trait(trait_item) = item {
                assert_eq!(trait_item.ident.to_string(), "Name");
                assert_eq!(trait_item.items.len(), 1);

                let fn_item = trait_item.items.first().unwrap();
                let fn_item = match fn_item {
                    syn::TraitItem::Fn(m) => m,
                    _ => panic!("Expected method"),
                };

                assert_eq!(fn_item.sig.ident.to_string(), "name");
                assert_eq!(fn_item.sig.inputs.len(), 1);
                assert!(matches!(
                    &fn_item.sig.output,
                    syn::ReturnType::Type(_, ty)
                    if matches!(
                        ty.as_ref(), syn::Type::Reference(syn::TypeReference {
                            elem, ..
                        })
                        if matches!(
                            elem.as_ref(),
                            syn::Type::Path(syn::TypePath {
                                path, ..
                            })
                            if path.segments.len() == 1
                                && path.segments.first().unwrap().ident.to_string() == "String"))));
            }
        });
    }

    #[test]
    fn test_substruct_use() {
        let tokens: proc_macro2::TokenStream = parse_substruct_use(
            parse_quote!(root = Query, fields(id, name)),
            parse_quote!(
                fn print_name(query: _) -> String {
                    query.name().clone()
                }
            ),
        );

        println!("{:#?}", tokens.to_string());
    }
}

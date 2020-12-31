use ::proc_macro::TokenStream;
use ::quote::quote;
use ::syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, DeriveInput, Ident, NestedMeta, Token,
};

const PUBLISHES_ATTRIBUTE: &str = "publish";
const HANDLES_ATTRIBUTE: &str = "handle";

#[proc_macro_derive(Actor, attributes(publish, handle))]
pub fn actor_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    process_actor_derive(ast)
}

fn process_actor_derive(input: DeriveInput) -> TokenStream {
    let publishes = get_idents(PUBLISHES_ATTRIBUTE, &input);
    let handles = get_idents(HANDLES_ATTRIBUTE, &input);

    let name = &input.ident;

    TokenStream::from(quote! {
        impl ::yaaf::Actor for #name {
            type Publishes = (#(#publishes, )*);
            type Handles = (#(#handles, )*);
        }
        #(
        impl ::yaaf::HandlerRegistered<#handles> for #name {}
        )*

        #(
        impl ::yaaf::Publisher<#publishes> for #name {}
        )*
    })
}

#[proc_macro_derive(Source, attributes(publish))]
pub fn source_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    process_source_derive(ast)
}

fn process_source_derive(input: DeriveInput) -> TokenStream {
    let publishes = get_idents(PUBLISHES_ATTRIBUTE, &input);

    let name = &input.ident;
    TokenStream::from(quote! {
        impl ::yaaf::SourceMeta for #name {
            type Publishes = (#(#publishes, )*);
        }

        #(
        impl ::yaaf::Publisher<#publishes> for #name {}
        )*
    })
}

fn get_idents(label: &str, input: &DeriveInput) -> Vec<Ident> {
    let mut result = vec![];
    for att in input.attrs.iter().filter(|a| a.path.is_ident(label)) {
        result.extend(parse_attr(&att));
    }
    result
}

fn parse_attr(attr: &Attribute) -> Vec<Ident> {
    let list = attr
        .parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)
        .expect("failed to parse attribute");

    let mut result = vec![];
    for item in list {
        match item {
            ::syn::NestedMeta::Meta(m) => {
                result.push(m.path().get_ident().unwrap().clone());
            }
            ::syn::NestedMeta::Lit(_) => {
                panic!("Identifier required");
            }
        }
    }
    result
}

#![allow(unused)]

use std::{cell::LazyCell, collections::HashSet};

use kernel::Kernel;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use statement::Statement;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, Ident, ItemFn, Path, PathSegment,
    Token,
};

mod expression;
mod kernel;
mod operator;
mod statement;

const IR_PREFIX: &'static str = "::squarecl_core::ir::";
const IR_PATH: LazyCell<Path> = LazyCell::new(|| {
    let span = Span::call_site();
    let mut path = Path::from(format_ident!("squarecl_core"));
    path.segments.push(format_ident!("ir").into());
    path.leading_colon = Some(Token![::](span));
    path
});

pub(crate) fn prefix_ir(ident: Ident) -> Path {
    let mut path = IR_PATH.clone();
    path.segments.push(ident.into());
    path
}

struct Args {
    /// This would hold launch, launch_unchecked
    options: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // If more complex parsing is needed, it would go here.
        let acceptable_values = ["launch", "launch_unchecked"];
        let options: Result<HashSet<Ident>, _> =
            Punctuated::<Ident, Token![,]>::parse_terminated(input)?
                .into_iter()
                .map(|ident| {
                    if acceptable_values.contains(&ident.to_string().as_str()) {
                        Ok(ident)
                    } else {
                        Err(syn::Error::new_spanned(
                            ident,
                            "Only `launch` or `launch_unchecked` are allowed.",
                        ))
                    }
                })
                .collect();
        Ok(Args { options: options? })
    }
}

#[proc_macro_attribute]
pub fn square(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let in_2 = input.clone();
    let kernel = parse_macro_input!(in_2 as Kernel);
    let function = parse_macro_input!(input as ItemFn);

    TokenStream::from(quote! {
        #function
        #kernel
    })
}

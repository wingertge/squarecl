use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, visit::Visit, FnArg, Ident, ItemFn, Pat, PatType, Visibility};

use crate::{prefix_ir, statement::Statement};

pub struct Kernel {
    visibility: Visibility,
    name: Ident,
    parameters: Vec<Ident>,
    statements: Vec<Statement>,
}

impl Parse for Kernel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let function: ItemFn = input.parse()?;
        let name = function.sig.ident;
        let vis = function.vis;
        let param_names = function
            .sig
            .inputs
            .into_iter()
            .map(|input| match &input {
                // Deref patterns please...
                FnArg::Typed(PatType { pat, .. }) => match &**pat {
                    Pat::Ident(ident) => Ok(ident.ident.clone()),
                    _ => Err(syn::Error::new_spanned(
                        input,
                        "Unsupported input for kernel",
                    )),
                },
                _ => Err(syn::Error::new_spanned(
                    input,
                    "Unsupported input for kernel",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let statements = function
            .block
            .stmts
            .into_iter()
            .map(|statement| Statement::from_stmt(statement))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Kernel {
            visibility: vis,
            name,
            parameters: param_names,
            statements,
        })
    }
}

impl ToTokens for Kernel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.visibility;
        let name = &self.name;
        let statements = &self.statements;
        let statement_ty = prefix_ir(format_ident!("Statement"));
        tokens.extend(quote! {
            #vis mod #name {
                pub fn expand
                /* Const generics could be used to supplement comptime */
                (/* Comptime values would go here */) -> Vec<#statement_ty> {
                    [
                        #(#statements),*
                    ].into_iter().collect()
                }
            }
        });
    }
}

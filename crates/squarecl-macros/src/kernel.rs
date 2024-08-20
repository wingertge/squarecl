use std::cell::RefCell;

use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, spanned::Spanned, visit::Visit, FnArg, Ident, ItemFn, Pat, PatType, Type,
    Visibility,
};

use crate::{expression::generate_var, prefix_ir, scope::Context, statement::Statement};

pub struct Kernel {
    visibility: Visibility,
    name: Ident,
    parameters: Vec<(Ident, Type)>,
    statements: Vec<Statement>,

    context: RefCell<Context>,
}

impl Parse for Kernel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut context = Context::default();

        let function: ItemFn = input.parse()?;
        let name = function.sig.ident;
        let vis = function.vis;
        let parameters = function
            .sig
            .inputs
            .into_iter()
            .map(|input| match &input {
                FnArg::Typed(arg) => Ok(arg.clone()),
                _ => Err(syn::Error::new_spanned(
                    input,
                    "Unsupported input for kernel",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let variables = parameters
            .into_iter()
            .map(|input| -> syn::Result<(Ident, Type)> {
                let ty = *input.ty;
                let ident = match *input.pat {
                    Pat::Ident(ident) => ident.ident,
                    input => Err(syn::Error::new_spanned(
                        input,
                        "kernel input should be ident",
                    ))?,
                };
                Ok((ident, ty))
            })
            .collect::<Result<Vec<_>, _>>()?;

        context.extend(
            variables
                .iter()
                .cloned()
                .map(|(ident, ty)| (ident, Some(ty))),
        );
        context.push_scope(); // Push function local scope

        let statements = function
            .block
            .stmts
            .into_iter()
            .map(|statement| Statement::from_stmt(statement, &mut context))
            .collect::<Result<Vec<_>, _>>()?;

        context.pop_scope(); // Pop function local scope

        Ok(Kernel {
            visibility: vis,
            name,
            parameters: variables,
            statements,
            context: RefCell::new(context),
        })
    }
}

impl ToTokens for Kernel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.visibility;
        let name = &self.name;
        let global_vars = self.context.borrow().current_scope().generate_vars();
        let statements = &self.statements;
        let statement_ty = prefix_ir(format_ident!("Statement"));
        let input_checks = self
            .parameters
            .iter()
            .map(|(_, ty)| {
                let span = ty.span();
                let check = prefix_ir(format_ident!("assert_valid_type"));
                quote_spanned! {span=>
                    #check::<#ty>();
                }
            })
            .collect::<Vec<_>>();
        tokens.extend(quote! {
            #vis mod #name {
                use super::*;

                fn __check_inputs() {
                    #(#input_checks)*
                }

                #[allow(unused_braces)]
                pub fn expand
                /* Const generics could be used to supplement comptime */
                (/* Comptime values would go here */) -> Vec<#statement_ty> {
                    #(#global_vars)*
                    {
                        let mut __statements = Vec::new();
                        #(#statements)*
                        __statements
                    }
                }
            }
        });
    }
}

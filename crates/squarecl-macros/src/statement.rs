use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident, Path, Stmt};

use crate::{expression::Expression, prefix_ir};

pub enum Statement {
    Local {
        variable: Box<Expression>,
        mutable: bool,
        init: Option<Box<Expression>>,
        span: Span,
    },
    Expression {
        expression: Box<Expression>,
        terminated: bool,
        span: Span,
    },
}

impl Statement {
    pub fn from_stmt(stmt: Stmt) -> syn::Result<Self> {
        let statement = match stmt {
            Stmt::Local(local) => {
                let span = local.span();
                let (path, mutable) = match local.pat {
                    syn::Pat::Ident(ident) => (Path::from(ident.ident), ident.mutability.is_some()),
                    _ => Err(syn::Error::new_spanned(local.clone(), "Unsupported local"))?,
                };
                let init = local
                    .init
                    .map(|init| Expression::from_expr(*init.expr))
                    .transpose()?
                    .map(Box::new);
                Self::Local {
                    variable: Box::new(Expression::Variable { name: path, span }),
                    mutable,
                    init,
                    span,
                }
            }
            Stmt::Expr(expr, semi) => Statement::Expression {
                terminated: semi.is_some(),
                span: expr.span(),
                expression: Box::new(Expression::from_expr(expr)?),
            },
            stmt => Err(syn::Error::new_spanned(stmt, "Unsupported statement"))?,
        };
        Ok(statement)
    }
}

impl ToTokens for Statement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let statement = prefix_ir(format_ident!("Statement"));
        let expr = prefix_ir(format_ident!("Expr"));

        let out = match self {
            Statement::Local {
                variable,
                mutable,
                init,
                span,
            } => {
                let span = span.clone();
                let init = if let Some(init) = init {
                    quote![Some(Box::new(#expr::expression_untyped(&#init)))]
                } else {
                    quote![None]
                };

                quote_spanned! {span=>
                    #statement::Local {
                        variable: Box::new(#expr::expression_untyped(&#variable)),
                        mutable: #mutable,
                        init: #init
                    }
                }
            }
            Statement::Expression {
                expression,
                terminated,
                span,
            } => {
                let span = span.clone();
                if *terminated {
                    quote_spanned! {span=>
                        #statement::Expression {
                            expression: Box::new(#expr::expression_untyped(&#expression))
                        }
                    }
                } else {
                    quote_spanned! {span=>
                        #statement::ImplicitReturn {
                            expression: Box::new(#expr::expression_untyped(&#expression))
                        }
                    }
                }
            }
        };

        tokens.extend(out);
    }
}

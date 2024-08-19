use proc_macro2::Span;
use quote::{format_ident, quote_spanned, ToTokens};
use syn::{
    parse::Parse,
    spanned::Spanned,
    visit::{visit_expr, Visit},
    Expr, Ident, Lit, Pat, Path,
};

use crate::{
    operator::{parse_binop, parse_unop, Operator},
    prefix_ir,
};

pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
        span: Span,
    },
    Unary {
        input: Box<Expression>,
        operator: Operator,
        span: Span,
    },
    Variable {
        name: Path,
        span: Span,
    },
    Literal {
        value: Lit,
        span: Span,
    },
    Assigment {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn from_expr(expr: Expr) -> syn::Result<Self> {
        let result = match expr {
            Expr::Assign(assign) => Expression::Assigment {
                span: assign.span(),
                left: Box::new(Self::from_expr(*assign.left)?),
                right: Box::new(Self::from_expr(*assign.right)?),
            },
            Expr::Binary(binary) => Expression::Binary {
                span: binary.span(),
                left: Box::new(Self::from_expr(*binary.left)?),
                operator: parse_binop(&binary.op)?,
                right: Box::new(Self::from_expr(*binary.right)?),
            },
            Expr::Lit(literal) => Expression::Literal {
                span: literal.span(),
                value: literal.lit,
            },
            Expr::Path(path) => Expression::Variable {
                span: path.span(),
                name: path.path,
            },
            Expr::Unary(unary) => Expression::Unary {
                span: unary.span(),
                input: Box::new(Self::from_expr(*unary.expr)?),
                operator: parse_unop(&unary.op)?,
            },
            _ => Err(syn::Error::new_spanned(expr, "Unsupported expression"))?,
        };
        Ok(result)
    }
}

impl ToTokens for Expression {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // TODO: Keep track of types on scopes instead of hardcoding u32

        let out = match self {
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("{}Expr", operator.to_string()));
                let ty_bin = prefix_ir(format_ident!("BinaryOp"));
                quote_spanned! {span=>
                    #ty(#ty_bin {
                        left: Box::new(#left),
                        right: Box::new(#right),
                        _out: ::core::marker::PhantomData,
                    })
                }
            }
            Expression::Unary {
                input,
                operator,
                span,
            } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("{}Expr", operator.to_string()));
                let ty_un = prefix_ir(format_ident!("UnaryOp"));
                quote_spanned! {span=>
                    #ty(#ty_un {
                        input: Box::new(#input),
                        _out: ::core::marker::PhantomData,
                    })
                }
            }
            Expression::Variable { name, span } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("Variable"));
                let name = name.to_token_stream().to_string();
                quote_spanned! {span=>
                    #ty::<u32> {
                        name: #name,
                        _type: ::core::marker::PhantomData
                    }
                }
            }
            Expression::Literal { value, span } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("Literal"));
                quote_spanned! {span=>
                    #ty {
                        value: #value
                    }
                }
            }
            Expression::Assigment { left, right, span } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("Assignment"));
                quote_spanned! {span=>
                    #ty {
                        left: Box::new(#left),
                        right: Box::new(#right)
                    }
                }
            }
        };

        tokens.extend(out);
    }
}

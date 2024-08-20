use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident, Pat, Path, Stmt, Type};

use crate::{
    expression::{generate_var, Expression},
    ir_type, prefix_ir,
    scope::Context,
};

pub enum Statement {
    Local {
        left: Box<Expression>,
        init: Option<Box<Expression>>,
        mutable: bool,
        ty: Option<Type>,
        span: Span,
    },
    Expression {
        expression: Box<Expression>,
        terminated: bool,
        span: Span,
    },
}

impl Statement {
    pub fn from_stmt(stmt: Stmt, context: &mut Context) -> syn::Result<Self> {
        let statement = match stmt {
            Stmt::Local(local) => {
                let span = local.span();
                let (ident, ty, mutable) = local_pat(local.pat)?;
                let init = local
                    .init
                    .map(|init| Expression::from_expr(*init.expr, context))
                    .transpose()?
                    .map(Box::new);
                let init_ty = init.as_ref().and_then(|init| init.ty());

                let variable = Box::new(Expression::Variable {
                    name: ident.clone(),
                    span: span.clone(),
                    ty: ty.clone(),
                });

                context.push_variable(ident, ty.clone());
                Self::Local {
                    left: variable,
                    init,
                    mutable,
                    ty,
                    span,
                }
            }
            Stmt::Expr(expr, semi) => Statement::Expression {
                terminated: semi.is_some(),
                span: expr.span(),
                expression: Box::new(Expression::from_expr(expr, context)?),
            },
            stmt => Err(syn::Error::new_spanned(stmt, "Unsupported statement"))?,
        };
        Ok(statement)
    }
}

fn local_pat(pat: Pat) -> syn::Result<(Ident, Option<Type>, bool)> {
    let res = match pat {
        Pat::Ident(ident) => (ident.ident, None, ident.mutability.is_some()),
        Pat::Type(pat) => {
            let ty = *pat.ty;
            let (ident, _, mutable) = local_pat(*pat.pat)?;
            (ident, Some(ty), mutable)
        }
        pat => Err(syn::Error::new_spanned(
            pat.clone(),
            format!("Unsupported local pat: {pat:?}"),
        ))?,
    };
    Ok(res)
}

impl ToTokens for Statement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let statement = prefix_ir(format_ident!("Statement"));
        let expr = prefix_ir(format_ident!("Expr"));

        let out = match self {
            Statement::Local {
                left,
                init,
                mutable,
                span,
                ty,
            } => {
                let span = span.clone();

                let name = match &**left {
                    Expression::Variable { name, .. } => name,
                    Expression::Init { left, .. } => match &**left {
                        Expression::Variable { name, .. } => name,
                        _ => panic!("Init left is always variable"),
                    },
                    _ => panic!("Local is always variable or init"),
                };
                // Separate init and declaration in case initializer uses an identically named
                // variable that would be overwritten by the declaration.
                let initializer = init.as_ref().map(|init| quote![let __init = #init;]);
                let left = if let Some(init) = init {
                    let span = span.clone();
                    let init_ty = ir_type("Initializer");
                    quote_spanned! {span=>
                        #init_ty {
                            left: Box::new(#name),
                            right: Box::new(__init)
                        }
                    }
                } else {
                    quote![Box::new(#name)]
                };
                let variable = generate_var(name, ty, span);
                let variable_decl = quote_spanned! {span=>
                    let #name = #variable;
                };

                let ty = if let Some(ty) = ty {
                    let span = ty.span();
                    let sq_type = prefix_ir(format_ident!("SquareType"));
                    quote_spanned! {span=>
                        Some(<#ty as #sq_type>::ir_type())
                    }
                } else {
                    quote![None]
                };

                quote_spanned! {span=>
                    #initializer
                    #variable_decl
                    __statements.push({
                            #statement::Local {
                            variable: Box::new(#expr::expression_untyped(&#left)),
                            mutable: #mutable,
                            ty: #ty
                        }
                    });
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
                        __statements.push(#statement::Expression {
                            expression: Box::new(#expr::expression_untyped(&#expression))
                        });
                    }
                } else {
                    quote_spanned! {span=>
                        __statements.push(#statement::ImplicitReturn {
                            expression: Box::new(#expr::expression_untyped(&#expression))
                        });
                    }
                }
            }
        };

        tokens.extend(out);
    }
}

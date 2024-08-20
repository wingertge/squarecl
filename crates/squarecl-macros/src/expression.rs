use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse,
    spanned::Spanned,
    visit::{visit_expr, Visit},
    Expr, Ident, Lit, Pat, Path, Type,
};

use crate::{
    ir_type,
    operator::{parse_binop, parse_unop, Operator},
    prefix_ir,
    scope::Context,
};

pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
        ty: Option<Type>,
        span: Span,
    },
    Unary {
        input: Box<Expression>,
        operator: Operator,
        ty: Option<Type>,
        span: Span,
    },
    Variable {
        name: Ident,
        ty: Option<Type>,
        span: Span,
    },
    Literal {
        value: Lit,
        ty: Type,
        span: Span,
    },
    Assigment {
        left: Box<Expression>,
        right: Box<Expression>,
        ty: Option<Type>,
        span: Span,
    },
    Init {
        left: Box<Expression>,
        right: Box<Expression>,
        ty: Option<Type>,
        span: Span,
    },
    /// Tokens not relevant to parsing
    Verbatim {
        tokens: TokenStream,
    },
}

impl Expression {
    pub fn from_expr(expr: Expr, context: &mut Context) -> syn::Result<Self> {
        let result = match expr {
            Expr::Assign(assign) => {
                let span = assign.span();
                let right = Self::from_expr(*assign.right, context)?;
                Expression::Assigment {
                    span,
                    ty: right.ty(),
                    left: Box::new(Self::from_expr(*assign.left, context)?),
                    right: Box::new(right),
                }
            }
            Expr::Binary(binary) => {
                let span = binary.span();
                let left = Self::from_expr(*binary.left, context)?;
                let right = Self::from_expr(*binary.right, context)?;
                let ty = left.ty().or(right.ty());
                Expression::Binary {
                    span,
                    left: Box::new(left),
                    operator: parse_binop(&binary.op)?,
                    right: Box::new(right),
                    ty,
                }
            }
            Expr::Lit(literal) => {
                let ty = lit_ty(&literal.lit)?;
                Expression::Literal {
                    span: literal.span(),
                    value: literal.lit,
                    ty,
                }
            }
            Expr::Path(path) => {
                let variable = path
                    .path
                    .get_ident()
                    .and_then(|ident| Some((ident.clone(), context.variable_type(ident)?)));
                if let Some((ident, ty)) = variable {
                    Expression::Variable {
                        span: path.span(),
                        name: ident.clone(),
                        ty,
                    }
                } else {
                    // If it's not in the scope, it's not a managed local variable. Treat it as an
                    // external value like a Rust `const`
                    Expression::Verbatim {
                        tokens: quote![#path],
                    }
                }
            }
            Expr::Unary(unary) => {
                let span = unary.span();
                let input = Self::from_expr(*unary.expr, context)?;
                let ty = input.ty();
                Expression::Unary {
                    span,
                    input: Box::new(input),
                    operator: parse_unop(&unary.op)?,
                    ty,
                }
            }
            _ => Err(syn::Error::new_spanned(expr, "Unsupported expression"))?,
        };
        Ok(result)
    }

    pub fn ty(&self) -> Option<Type> {
        match self {
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Unary { ty, .. } => ty.clone(),
            Expression::Variable { ty, .. } => ty.clone(),
            Expression::Literal { ty, .. } => Some(ty.clone()),
            Expression::Assigment { ty, .. } => ty.clone(),
            Expression::Verbatim { .. } => None,
            Expression::Init { ty, .. } => ty.clone(),
        }
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
                ..
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
                ..
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
            Expression::Variable { name, span, ty } => {
                let span = span.clone();
                quote_spanned! {span=>
                    #name
                }
            }
            Expression::Literal { value, span, ty } => {
                let span = span.clone();
                let ir_ty = prefix_ir(format_ident!("Literal"));
                quote_spanned! {span=>
                    #ir_ty {
                        value: #value
                    }
                }
            }
            Expression::Assigment {
                left, right, span, ..
            } => {
                let span = span.clone();
                let ty = prefix_ir(format_ident!("Assignment"));
                quote_spanned! {span=>
                    #ty {
                        left: Box::new(#left),
                        right: Box::new(#right)
                    }
                }
            }
            Expression::Init {
                left,
                right,
                ty,
                span,
            } => {
                let span = span.clone();
                let ir_type = ir_type("Initializer");
                let ty = right.ty().map(|ty| quote![::<#ty>]);
                quote_spanned! {span=>
                    #ir_type #ty {
                        left: Box::new(#left),
                        right: Box::new(#right)
                    }
                }
            }
            Expression::Verbatim { tokens } => {
                let span = tokens.span();
                let ty = prefix_ir(format_ident!("Literal"));
                quote_spanned! {span=>
                    #ty {
                        value: #tokens
                    }
                }
            }
        };

        tokens.extend(out);
    }
}

fn lit_ty(lit: &Lit) -> syn::Result<Type> {
    let res = match lit {
        Lit::Int(int) => (!int.suffix().is_empty())
            .then(|| int.suffix())
            .map(|suffix| format_ident!("{suffix}"))
            .and_then(|ident| syn::parse2(quote![#ident]).ok())
            .unwrap_or_else(|| syn::parse2(quote![i32]).unwrap()),
        Lit::Float(float) => (!float.suffix().is_empty())
            .then(|| float.suffix())
            .map(|suffix| format_ident!("{suffix}"))
            .and_then(|ident| syn::parse2(quote![#ident]).ok())
            .unwrap_or_else(|| syn::parse2(quote![f32]).unwrap()),
        Lit::Bool(_) => todo!("Not supported yet"),
        lit => Err(syn::Error::new_spanned(
            lit,
            format!("Unsupported literal type: {lit:?}"),
        ))?,
    };
    Ok(res)
}

pub fn generate_var(name: &Ident, ty: &Option<Type>, span: Span) -> TokenStream {
    let var = prefix_ir(format_ident!("Variable"));
    let name = name.to_token_stream().to_string();
    let ty = ty.as_ref().map(|ty| {
        quote_spanned! {ty.span()=>
            ::<#ty>
        }
    });
    quote_spanned! {span=>
        #var #ty {
            name: #name,
            _type: ::core::marker::PhantomData
        }
    }
}

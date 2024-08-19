use std::fmt::Display;

use derive_more::derive::Display;
use syn::{visit::Visit, BinOp, UnOp};

#[derive(Debug, Clone, Copy, Display)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Deref,
    Not,
    Neg,
}

pub fn parse_binop(op: &BinOp) -> syn::Result<Operator> {
    let op = match op {
        BinOp::Add(_) => Operator::Add,
        BinOp::Sub(_) => Operator::Sub,
        BinOp::Mul(_) => Operator::Mul,
        BinOp::Div(_) => Operator::Div,
        _ => Err(syn::Error::new_spanned(op, "Unsupported operator"))?,
    };
    Ok(op)
}

pub fn parse_unop(op: &UnOp) -> syn::Result<Operator> {
    let op = match op {
        UnOp::Deref(_) => Operator::Deref,
        UnOp::Not(_) => Operator::Not,
        UnOp::Neg(_) => Operator::Neg,
        _ => Err(syn::Error::new_spanned(op, "Unsupported operator"))?,
    };
    Ok(op)
}

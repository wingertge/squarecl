use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, Deref, Div, Mul, Neg, Not, Sub},
};

use super::operator::Operator;

pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Unary {
        input: Box<Expression>,
        operator: Operator,
    },
    Variable {
        name: String,
    },
    Literal {
        // Stringified value for outputting directly to generated code
        value: String,
    },
    Assigment {
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

pub trait Expr {
    type Output;

    fn expression_untyped(&self) -> Expression;
}

pub struct BinaryOp<TLeft, TRight, TOut> {
    pub left: Box<dyn Expr<Output = TLeft>>,
    pub right: Box<dyn Expr<Output = TRight>>,
    pub _out: PhantomData<TOut>,
}

pub struct UnaryOp<TIn, TOut> {
    pub input: Box<dyn Expr<Output = TIn>>,
    pub _out: PhantomData<TOut>,
}

macro_rules! bin_op {
    ($name:ident, $trait:ident, $operator:path) => {
        pub struct $name<TLeft, TRight, TOut>(pub BinaryOp<TLeft, TRight, TOut>)
        where
            TLeft: $trait<TRight, Output = TOut>;

        impl<TLeft, TRight, TOut> Expr for $name<TLeft, TRight, TOut>
        where
            TLeft: $trait<TRight, Output = TOut>,
        {
            type Output = TOut;

            fn expression_untyped(&self) -> Expression {
                Expression::Binary {
                    left: Box::new(self.0.left.expression_untyped()),
                    right: Box::new(self.0.right.expression_untyped()),
                    operator: $operator,
                }
            }
        }
    };
}

macro_rules! unary_op {
    ($name:ident, $trait:ident, $operator:path, $target:ident) => {
        pub struct $name<TIn: $trait<$target = TOut>, TOut>(pub UnaryOp<TIn, TOut>);

        impl<TIn: $trait<$target = TOut>, TOut> Expr for $name<TIn, TOut> {
            type Output = TOut;

            fn expression_untyped(&self) -> Expression {
                Expression::Unary {
                    input: Box::new(self.0.input.expression_untyped()),
                    operator: $operator,
                }
            }
        }
    };
}

bin_op!(AddExpr, Add, Operator::Add);
bin_op!(SubExpr, Sub, Operator::Sub);
bin_op!(MulExpr, Mul, Operator::Mul);
bin_op!(DivExpr, Div, Operator::Div);

unary_op!(NotExpr, Not, Operator::Not, Output);
unary_op!(NegExpr, Neg, Operator::Neg, Output);
unary_op!(DerefExpr, Deref, Operator::Deref, Target);

pub struct Variable<T> {
    pub name: &'static str,
    pub _type: PhantomData<T>,
}

impl<T> Expr for Variable<T> {
    type Output = T;

    fn expression_untyped(&self) -> Expression {
        Expression::Variable {
            name: self.name.to_string(),
        }
    }
}

pub struct Literal<T: Display> {
    pub value: T,
}

impl<T: Display> Expr for Literal<T> {
    type Output = T;

    fn expression_untyped(&self) -> Expression {
        Expression::Literal {
            value: self.value.to_string(),
        }
    }
}

pub struct Assignment<TLeft, TRight> {
    pub left: Box<dyn Expr<Output = TLeft>>,
    pub right: Box<dyn Expr<Output = TRight>>,
}

impl<TLeft, TRight> Expr for Assignment<TLeft, TRight> {
    type Output = ();

    fn expression_untyped(&self) -> Expression {
        Expression::Assigment {
            left: Box::new(self.left.expression_untyped()),
            right: Box::new(self.right.expression_untyped()),
        }
    }
}

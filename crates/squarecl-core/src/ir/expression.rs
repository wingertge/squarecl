use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, Deref, Div, Mul, Neg, Not, Sub},
};

use super::{operator::Operator, IRType, SquareType};

pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
        ty: IRType,
    },
    Unary {
        input: Box<Expression>,
        operator: Operator,
        ty: IRType,
    },
    Variable {
        name: String,
        ty: IRType,
    },
    Literal {
        // Stringified value for outputting directly to generated code
        value: String,
        ty: IRType,
    },
    Assigment {
        left: Box<Expression>,
        right: Box<Expression>,
        ty: IRType,
    },
    /// Local variable initializer
    Init {
        left: Box<Expression>,
        right: Box<Expression>,
        ty: IRType,
    },
}

impl Expression {
    pub fn ir_type(&self) -> IRType {
        match self {
            Expression::Binary { ty, .. } => *ty,
            Expression::Unary { ty, .. } => *ty,
            Expression::Variable { ty, .. } => *ty,
            Expression::Literal { ty, .. } => *ty,
            Expression::Assigment { ty, .. } => *ty,
            Expression::Init { ty, .. } => *ty,
        }
    }
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
        pub struct $name<TLeft, TRight, TOut: SquareType>(pub BinaryOp<TLeft, TRight, TOut>)
        where
            TLeft: $trait<TRight, Output = TOut>;

        impl<TLeft, TRight, TOut: SquareType> Expr for $name<TLeft, TRight, TOut>
        where
            TLeft: $trait<TRight, Output = TOut>,
        {
            type Output = TOut;

            fn expression_untyped(&self) -> Expression {
                Expression::Binary {
                    left: Box::new(self.0.left.expression_untyped()),
                    right: Box::new(self.0.right.expression_untyped()),
                    operator: $operator,
                    ty: <TOut as SquareType>::ir_type(),
                }
            }
        }
    };
}

macro_rules! unary_op {
    ($name:ident, $trait:ident, $operator:path, $target:ident) => {
        pub struct $name<TIn: $trait<$target = TOut>, TOut>(pub UnaryOp<TIn, TOut>);

        impl<TIn: $trait<$target = TOut>, TOut: SquareType> Expr for $name<TIn, TOut> {
            type Output = TOut;

            fn expression_untyped(&self) -> Expression {
                Expression::Unary {
                    input: Box::new(self.0.input.expression_untyped()),
                    operator: $operator,
                    ty: <TOut as SquareType>::ir_type(),
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

#[derive(Clone, Copy, Debug)]
pub struct Variable<T: SquareType> {
    pub name: &'static str,
    pub _type: PhantomData<T>,
}

impl<T: SquareType> Expr for Variable<T> {
    type Output = T;

    fn expression_untyped(&self) -> Expression {
        Expression::Variable {
            name: self.name.to_string(),
            ty: <T as SquareType>::ir_type(),
        }
    }
}

pub struct Literal<T: Display + SquareType> {
    pub value: T,
}

impl<T: Display + SquareType> Expr for Literal<T> {
    type Output = T;

    fn expression_untyped(&self) -> Expression {
        Expression::Literal {
            value: self.value.to_string(),
            ty: <T as SquareType>::ir_type(),
        }
    }
}

pub struct Assignment<T: SquareType> {
    pub left: Box<dyn Expr<Output = T>>,
    pub right: Box<dyn Expr<Output = T>>,
}

impl<T: SquareType> Expr for Assignment<T> {
    type Output = ();

    fn expression_untyped(&self) -> Expression {
        Expression::Assigment {
            left: Box::new(self.left.expression_untyped()),
            right: Box::new(self.right.expression_untyped()),
            ty: <T as SquareType>::ir_type(),
        }
    }
}

pub struct Initializer<T: SquareType> {
    pub left: Box<dyn Expr<Output = T>>,
    pub right: Box<dyn Expr<Output = T>>,
}

impl<T: SquareType> Expr for Initializer<T> {
    type Output = T;

    fn expression_untyped(&self) -> Expression {
        Expression::Init {
            left: Box::new(self.left.expression_untyped()),
            right: Box::new(self.right.expression_untyped()),
            ty: <T as SquareType>::ir_type(),
        }
    }
}

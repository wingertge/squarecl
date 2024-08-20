use super::{Expression, IRType};

pub enum Statement {
    Local {
        variable: Box<Expression>,
        mutable: bool,
        ty: Option<IRType>,
    },
    Expression {
        expression: Box<Expression>,
    },
    ImplicitReturn {
        expression: Box<Expression>,
    },
}

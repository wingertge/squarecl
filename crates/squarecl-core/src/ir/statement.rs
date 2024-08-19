use super::Expression;

pub enum Statement {
    Local {
        variable: Box<Expression>,
        mutable: bool,
        init: Option<Box<Expression>>,
    },
    Expression {
        expression: Box<Expression>,
    },
    ImplicitReturn {
        expression: Box<Expression>,
    },
}

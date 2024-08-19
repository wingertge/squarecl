use std::fmt::{Display, Error, Formatter};

use derive_more::derive::Deref;
use squarecl_core::ir::{Expression, Operator, Statement};

pub struct WgpuKernel(pub Vec<Statement>);
#[derive(Deref)]
struct WgpuStatement<'a>(&'a Statement);

#[derive(Deref)]
struct WgpuExpression<'a>(&'a Box<Expression>);

#[derive(Deref)]
struct WgpuOperator<'a>(&'a Operator);

fn e(expr: &Box<Expression>) -> WgpuExpression<'_> {
    WgpuExpression(expr)
}

fn o(expr: &Operator) -> WgpuOperator<'_> {
    WgpuOperator(expr)
}

impl Display for WgpuKernel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn main(a: u32, b: u32) {{")?;
        for statement in &self.0 {
            let statement = WgpuStatement(statement);
            write!(f, "{statement}")?;
        }
        writeln!(f, "}}")
    }
}

impl<'a> Display for WgpuStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.0 {
            Statement::Local {
                variable,
                init,
                mutable,
            } => {
                let variable = e(variable);
                let keyword = if *mutable { "var" } else { "let" };
                if let Some(init) = init {
                    let init = e(init);
                    writeln!(f, "{keyword} {variable} = {init};")
                } else {
                    writeln!(f, "{keyword} {variable}: u32;") // TODO: Type
                }
            }
            Statement::Expression { expression } => {
                let expression = e(expression);
                writeln!(f, "{expression};")
            }
            Statement::ImplicitReturn { expression } => {
                let expression = e(expression);
                writeln!(f, "return {expression};")
            }
        }
    }
}

impl<'a> Display for WgpuExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &**self.0 {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = e(left);
                let operator = o(operator);
                let right = e(right);
                write!(f, "{left} {operator} {right}",)
            }
            Expression::Unary { input, operator } => {
                let input = e(input);
                let operator = o(operator);
                write!(f, "{operator}{input}")
            }
            Expression::Variable { name } => write!(f, "{name}"),
            Expression::Literal { value } => write!(f, "{value}u"), // TODO: Types
            Expression::Assigment { left, right } => {
                let left = e(left);
                let right = e(right);
                write!(f, "{left} = {right}")
            }
        }
    }
}

impl<'a> Display for WgpuOperator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Deref => write!(f, "*"),
            Operator::Not => write!(f, "!"),
            Operator::Neg => write!(f, "-"),
        }
    }
}

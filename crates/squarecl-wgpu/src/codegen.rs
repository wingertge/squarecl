use std::fmt::{Display, Error, Formatter};

use derive_more::derive::Deref;
use squarecl_core::ir::{Expression, IRType, Operator, Statement};

pub struct WgpuKernel(pub Vec<Statement>);
#[derive(Deref)]
struct WgpuStatement<'a>(&'a Statement);

#[derive(Deref)]
struct WgpuExpression<'a>(&'a Box<Expression>);

#[derive(Deref)]
struct WgpuOperator<'a>(&'a Operator);

struct WgpuType(IRType);

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
                mutable,
                ty,
            } => {
                let variable = e(variable);
                let keyword = if *mutable { "var" } else { "let" };
                match &**variable.0 {
                    Expression::Init { left, right, .. } => {
                        let ty = ty
                            .map(WgpuType)
                            .map(|ty| format!(": {ty}"))
                            .unwrap_or("".to_string());
                        let left = e(left);
                        let right = e(right);
                        writeln!(f, "{keyword} {left}{ty} = {right};")
                    }
                    _ => {
                        // Prefer explicit type
                        let ty = WgpuType(ty.unwrap_or(variable.ir_type()));
                        writeln!(f, "{keyword} {variable}: {ty};") // TODO: Type
                    }
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
                ..
            } => {
                let left = e(left);
                let operator = o(operator);
                let right = e(right);
                write!(f, "{left} {operator} {right}",)
            }
            Expression::Unary {
                input, operator, ..
            } => {
                let input = e(input);
                let operator = o(operator);
                write!(f, "{operator}{input}")
            }
            Expression::Variable { name, .. } => write!(f, "{name}"),
            Expression::Literal { value, ty } => format_lit(f, value, ty), // TODO: Types
            Expression::Assigment { left, right, .. } => {
                let left = e(left);
                let right = e(right);
                write!(f, "{left} = {right}")
            }
            Expression::Init { .. } => {
                panic!("Init should be handled by `Statement::Local`");
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

impl Display for WgpuType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ty = match self.0 {
            IRType::Int(32) => "i32",
            IRType::UInt(32) => "u32",
            IRType::Float(16) => "f32",
            t => panic!("Unsupported data type {:?}", t),
        };
        write!(f, "{ty}")
    }
}

fn format_lit(f: &mut Formatter<'_>, value: &str, ty: &IRType) -> Result<(), Error> {
    let suffix = match ty {
        IRType::Int(32) => "i",
        IRType::UInt(32) => "u",
        IRType::Float(32) => "f",
        t => panic!("Unsupported data type {:?}", t),
    };
    write!(f, "{value}{suffix}")
}

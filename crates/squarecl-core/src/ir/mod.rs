mod expression;
mod operator;
mod statement;
mod types;

pub use expression::*;
pub use operator::*;
pub use statement::*;
pub use types::*;

pub fn assert_valid_type<T: SquareType>() {}

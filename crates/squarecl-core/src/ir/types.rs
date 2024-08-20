#[derive(Debug, Clone, Copy)]
pub enum IRType {
    Int(usize),
    UInt(usize),
    Float(usize),
    Unit,
}

pub trait SquareType {
    fn ir_type() -> IRType;
}

macro_rules! primitive {
    ($primitive:ident, $var_type:expr) => {
        impl SquareType for $primitive {
            fn ir_type() -> IRType {
                $var_type
            }
        }
    };
}

primitive!(i16, IRType::Int(16));
primitive!(i32, IRType::Int(32));
primitive!(i64, IRType::Int(64));
primitive!(u16, IRType::UInt(16));
primitive!(u32, IRType::UInt(32));
primitive!(u64, IRType::UInt(64));
primitive!(f32, IRType::Float(32));
primitive!(f64, IRType::Float(64));

use squarecl_macros::square;

pub mod codegen;

const ALPHA: i32 = 10;

#[square]
pub fn test_kernel(a: i32, b: i32) {
    let mut d = 0;
    let a = a * b;
    let c = a + b + ALPHA;
    d = a / c + 2;
    let f = 2u32;
}

use squarecl_macros::square;

pub mod codegen;

const ALPHA: u32 = 10;

#[square]
pub fn test_kernel(a: u32, b: u32) {
    let mut d: u32 = 0;
    let a = a * b;
    let c = a + b + ALPHA;
    d = a / c + 2;
}

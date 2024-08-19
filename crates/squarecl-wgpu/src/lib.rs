use squarecl_macros::square;

pub mod codegen;

#[square]
pub fn test_kernel(a: u32, b: u32) {
    let mut d = 0;
    let a = a * b;
    let c = a + b;
    d = a / c + 2;
}

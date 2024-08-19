use squarecl_macros::square;

#[square]
fn test_kernel(a: u32, b: u32, mut d: u32) {
    let a = a * b;
    let c = a + b;
    d = a / c + 2;
}

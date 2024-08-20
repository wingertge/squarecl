use squarecl_statements::{codegen::WgpuKernel, test_kernel};

fn main() {
    let kernel = WgpuKernel(test_kernel::expand());
    let shader = format!("{}", kernel);
    std::fs::create_dir_all("out").unwrap();
    std::fs::write("out/kernel_statements.wgsl", shader).unwrap();
}

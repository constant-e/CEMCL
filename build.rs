fn main() {
    println!("cargo:rerun-if-changed=res");
    // slint ui files
    slint_build::compile("res/ui/cemcl.slint").unwrap();
}

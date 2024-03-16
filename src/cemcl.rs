use slint::{};

slint::include_modules!();

pub fn init() {
    let cemcl = CEMCL::new().expect("msg");
    slint::init_translations!(std::env::current_exe().unwrap().parent().unwrap().join("translations"));
    cemcl.run().expect("msg");
}
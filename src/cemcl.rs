use slint::{};

slint::include_modules!();

pub fn init() {
    let window = CEMCL::new().expect("[Error] cemcl: Couldn't create window");
    slint::init_translations!(std::env::current_exe().unwrap().parent().unwrap().join("translations"));
    window.run().expect("[Error] cemcl: Could't start.");
}
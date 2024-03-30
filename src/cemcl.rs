use log::{info};
use slint::{include_modules};

include_modules!();

struct Config {

}

fn load_config() {

}

pub fn init() {
    info!(target: "cemcl", "Start.");
    let window = CEMCL::new().expect("Couldn't create window.");
    slint::init_translations!(std::env::current_exe().unwrap().parent().unwrap().join("translations"));
    window.run().expect("Could't start.");

    println!("{}", window.get_acc_list_index());

    window.on_clicked_add_btn(|| {

    });
    window.on_clicked_edit_btn(|| {

    });
    window.on_clicked_settings_btn(|| {

    });
    window.on_clicked_start_btn(|| {

    })
}
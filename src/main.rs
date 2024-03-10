mod cemcl;
mod file_tools;
mod mc_core;

use gtk4::{Application, prelude::*};

fn main() {
    println!("[Info] cemcl: Program start.");
    // load cemcl ui
    let app = Application::builder().application_id("io.github.constante.cemcl").build();
    app.connect_activate(cemcl::build_ui);
    app.run();
}

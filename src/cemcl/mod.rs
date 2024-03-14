use gtk4::{Application, ApplicationWindow, Builder, prelude::*};

fn load_account() {

}

fn load_config() {

}

fn load_game() {

}

fn on_click_start_btn() {
    
}

pub fn build_ui(app: &Application) {
    let src = include_str!("res/cemcl.ui");
    let builder = Builder::from_string(src);
    let window = builder
        .object::<ApplicationWindow>("mainwindow")
        .expect("[Error] cemcl: Couldn't get window");
    window.set_application(Some(app));
    window.present();
}

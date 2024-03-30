mod cemcl;
mod file_tools;
mod mc_core;

use log::info;

fn main() {
    info!(target: "main", "Program start.");
    cemcl::init();
}

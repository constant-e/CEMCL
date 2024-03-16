use std::fmt::Debug;
use std::fs;
use std::path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=res");
    // slint ui files
    slint_build::compile("res/ui/cemcl.slint").unwrap();
    // translate files
    Command::new("msgfmt -V").output().expect("[Error] build: Counld't run msgfmt");
    if path::Path::new(&std::env::var("OUT_DIR").unwrap()).exists() {
        fs::remove_dir(&std::env::var("OUT_DIR").unwrap()).unwrap();
    }
    for p in path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/res/translate")).read_dir().unwrap() {
        if p.unwrap().file_type().unwrap().is_file() {
            let file_name = p.unwrap().file_name().into_string();

        }
    }
}
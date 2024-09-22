fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("qt".into());
    slint_build::compile_with_config("res/ui/appwindow.slint", config).unwrap();
}

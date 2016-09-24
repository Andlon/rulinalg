use std::path::PathBuf;

use compiletest::common::Mode;
use compiletest;

fn run_mode(mode: Mode, directory: &str) {
    let mut config = compiletest::default_config();
    config.mode = mode;
    config.src_base = PathBuf::from(format!("tests/compiletests/{}", directory));
    config.target_rustcflags = Some("-L target/debug -L target/debug/deps".to_string());

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode(Mode::CompileFail, "compile-fail");
}
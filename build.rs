use std::process::Command;

fn main() {
    Command::new("git")
        .args(["-C", "ooz", "apply", "-q", "../ooz.patch"])
        .status()
        .unwrap();
    cc::Build::new()
        .file("ooz/kraken.cpp")
        .flag("-Wno-sign-compare")
        .flag("-Wno-unused-variable")
        .flag("-Wno-unused-parameter")
        .compile("kraken");
    println!("cargo:rerun-if-changed=ooz/kraken.cpp");
}

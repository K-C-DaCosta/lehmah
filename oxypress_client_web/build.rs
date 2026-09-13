use oxypress_core::{log::ConsoleColors, loggy};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    check_if_trunk_installed();
    issue_stub_warning_if_applicable();
}

fn issue_stub_warning_if_applicable() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "wasm32" {
        println!("cargo:warning=oxypress_client_web is compiling as a stub on non-wasm32 targets.");
    }
}

fn check_if_trunk_installed() {
    let trunk_check = Command::new("trunk").arg("--version").output();
    match trunk_check {
        Ok(version) => {
            let version = String::from_utf8(version.stdout).expect("Version unparsable!");
            loggy!(
                ConsoleColors::YELLOW,
                "Trunk (version = {}) already exits. Skipping...",
                version.trim()
            );
        }
        Err(e) => {
            panic!(
                "Failed to find trunk executable: {}. Please install it using `cargo install trunk`.",
                e
            );
        }
    }
}

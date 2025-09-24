use oxypress_core::{log::ConsoleColors, loggy};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    install_trunk_if_not_installed();
}

fn install_trunk_if_not_installed() {
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

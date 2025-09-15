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
        Err(_) => {
            let status = Command::new("cargo")
                .arg("install")
                .arg("--locked")
                .arg("trunk") // For an optimized, production-ready build
                .status()
                .expect("Failed to execute trunk install");

            if !status.success() {
                panic!("Trunk install failed");
            } else {
                loggy!(ConsoleColors::GREEN, "Trunk installed with no Errors!");
            }
        }
    }
}

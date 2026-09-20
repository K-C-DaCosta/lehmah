use actix_web::web;

use oxypress_core::{log::ConsoleColors, loggy, prelude::*};
use serde::Deserialize;
use std::{fmt::Debug, fs, io, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub oxypress_web_root: PathBuf,
    pub oxypress_web_enable_logging: bool,
    pub oxypress_web_http_port: u32,
    pub oxypress_web_https_port: u32,
    pub oxypress_web_cert_directory: PathBuf,
}

pub mod routes;

pub fn initalize_oxypress_env_vars() {
    match std::env::var("OXYPRESS_WEB_ENV_FILE_DIR").ok() {
        Some(env_dir) => {
            dotenvy::from_path(env_dir).expect("Failed to read OXYPRESS_WEB_ENV_FILE_DIR.")
        }
        None => {
            loggy!(ConsoleColors::YELLOW, ".ENV file not found");
        }
    }
}

pub fn configure_tls(env_ctx: web::Data<EnvConfig>) -> rustls::ServerConfig {
    let cert_directory = &env_ctx.oxypress_web_cert_directory;

    let mut certs_file = io::BufReader::new(
        fs::File::open(cert_directory.join("./cert.pem")).expect("Failed to read cert.pem"),
    );
    let mut key_file = io::BufReader::new(
        fs::File::open(cert_directory.join("./key.pem")).expect("Failed to read key.pem"),
    );

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}

pub fn translate_to_physical<P: AsRef<std::path::Path>>(
    env_ctx: web::Data<EnvConfig>,
    path: P,
) -> std::path::PathBuf {
    env_ctx.oxypress_web_root.join(path)
}

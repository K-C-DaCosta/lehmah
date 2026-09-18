use actix_files::NamedFile;
use actix_web::{
    error, get, guard, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};

use oxypress_core::{log::ConsoleColors, loggy};
use std::{fs, io, path::PathBuf};

pub mod routes;

macro_rules! get_env_with_generic_expect {
    ($env_var:expr) => {
        std::env::var($env_var).expect(concat!(
            "Failed to read the enviroment variable: \"",
            $env_var,
            "\""
        ))
    };
}

pub fn initalize_oxypress_env_vars() {
    match std::env::var("OXYPRESS_WEB_ENV_FILE_DIR").ok() {
        Some(env_dir) => {
            dotenvy::from_path(env_dir).expect("Failed to read OXYPRESS_WEB_ENV_FILE_DIR.")
        }
        None => {
            loggy!(
                ConsoleColors::YELLOW,
                ".ENV file not found. `Makefile.toml` is the single source of truth for all env varibles"
            );
        }
    }
}

pub fn configure_tls() -> rustls::ServerConfig {
    let cert_directory = PathBuf::from(get_env_with_generic_expect!("OXYPRESS_WEB_CERT_DIRECTORY"));

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

pub fn translate_to_physical<P: AsRef<std::path::Path>>(path: P) -> std::path::PathBuf {
    let web_root = std::path::PathBuf::from(get_env_with_generic_expect!("OXYPRESS_WEB_ROOT"));
    web_root.join(path)
}

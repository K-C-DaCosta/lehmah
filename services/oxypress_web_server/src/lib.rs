use actix_files::NamedFile;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, error, get, guard, middleware::Logger,
    post, web,
};

use oxypress_core::{log::ConsoleColors, loggy};
use std::{fs, io};

const PHYSICAL_ROOT_DIR: &str = "./resources";
const DEFAULT_ENV_FILE: &str = "./.env.local.template";

pub mod routes;

pub fn initalize_oxypress_env_vars() {
    dotenvy::from_path(match std::env::var("OXYPRESS_ENV_DIR").ok() {
        Some(env_dir) => env_dir,
        None => {
            loggy!(
                ConsoleColors::YELLOW,
                "Default ENV file selected. Assuming local run."
            );
            String::from(DEFAULT_ENV_FILE)
        }
    })
    .unwrap();
}

pub fn configure_tls() -> rustls::ServerConfig {
    let untrusted_certs_dir = std::env::var("OXYPRESS_WEB_UNTRUSTED_CERT_DIRECTORY").ok();
    let trusted_certs_dir = std::env::var("OXYPRESS_WEB_TRUSTED_CERT_DIRECTORY").ok();

    let configured_directory = std::path::PathBuf::from(
        match (untrusted_certs_dir, trusted_certs_dir) {
            (None, None) => {
                panic!(
                    "At least one cert directory must be specified. Please make sure the env file is properly configured"
                );
            }
            (Some(untrusted_dir), None) => {
                loggy!(ConsoleColors::YELLOW, "untrusted_certs selected");
                untrusted_dir
            }
            (Some(untrusted_dir), Some(_)) => {
                loggy!(ConsoleColors::YELLOW, "untrusted_certs selected");
                untrusted_dir
            }
            (None, Some(trusted_dir)) => {
                loggy!(ConsoleColors::YELLOW, "trusted_certs selected");
                trusted_dir
            }
        },
    );

    let mut certs_file =
        io::BufReader::new(fs::File::open(configured_directory.join("./cert.pem")).unwrap());
    let mut key_file =
        io::BufReader::new(fs::File::open(configured_directory.join("./key.pem")).unwrap());

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


use actix_files::NamedFile;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, error, get, guard, middleware::Logger,
    post, web,
};
use oxypress_core::{loggy,log::*};
use std::{fs, io};
use crate::{DEFAULT_ENV_FILE,PHYSICAL_ROOT_DIR};

pub struct UserSessionContext {}

pub fn authenticate<Cb, CbOut, AuthOut>(_req: &HttpRequest, callback: Cb) -> CbOut
where
    Cb: FnOnce(UserSessionContext) -> CbOut,
    CbOut: Future<Output = AuthOut>,
{
    let auth_ctx = UserSessionContext {};
    // Gets cookie info from request, finds client session from request
    callback(auth_ctx) // callback does task now with user state in mind
}

#[get("/")]
async fn homepage(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    if let Some(val) = req.headers().get("host") {
        let host_value = val.to_str().unwrap();
        loggy!(ConsoleColors::YELLOW, "host header = {}", host_value);
    }
    loggy!(ConsoleColors::YELLOW, "homepage requested!");
    let path = translate_to_physical("homepage.html");
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
}

#[get("/")]
async fn blog_homepage(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    loggy!(ConsoleColors::YELLOW, "BLOG homepage requested!");
    let path = translate_to_physical("blog_homepage.html");
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
}

#[get("/{filename:.+\\.[a-zA-z]+}")]
async fn fetch_files_on_disk(
    req: HttpRequest,
    filename: web::Path<String>,
) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    loggy!(ConsoleColors::YELLOW, "file '{:?}' ", filename);
    let path = translate_to_physical(filename.as_str());
    loggy!(ConsoleColors::YELLOW, "path = {:?} requested!", path);
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
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

pub fn translate_to_physical<P: AsRef<std::path::Path>>(path: P) -> std::path::PathBuf {
    std::path::Path::new(PHYSICAL_ROOT_DIR).join(path)
}

#[test]
fn chrono_test() {
    print!("Chrono = {}", 123);
}

#[test]
fn backtrace_check() {
    fn recursive_thing(depth: i32) {
        if depth <= 0 {
            let bt = backtrace::Backtrace::new();
            println!("{:?}", bt);
            return;
        }
        recursive_thing(depth - 1);
    }
    recursive_thing(10);
}

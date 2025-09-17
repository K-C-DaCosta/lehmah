use actix_web::{App, HttpServer, guard, web};
use oxypress_core::{log::ConsoleColors, loggy};
use std::{fs, io};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let cwd = std::env::current_dir()?;
    std::env::set_current_dir(cwd.join("oxypress_server"))?;

    loggy!(
        ConsoleColors::YELLOW,
        "Servers CWD set to: {:?}",
        std::env::current_dir().unwrap(),
    );

    let mut certs_file =
        io::BufReader::new(fs::File::open("./generated_local_certs/cert.pem").unwrap());
    let mut key_file =
        io::BufReader::new(fs::File::open("./generated_local_certs/key.pem").unwrap());

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
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            // .wrap(Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T host=%{HOST}i"))
            .service(
                web::scope("")
                    .guard(guard::Host("local.khadeemdacosta.ca"))
                    .service(oxypress_server::homepage),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("www.khadeemdacosta.ca"))
                    .service(oxypress_server::homepage),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("blog.khadeemdacosta.ca"))
                    .service(oxypress_server::blog_homepage),
            )
            .service(oxypress_server::homepage)
            .service(oxypress_server::fetch_files_on_disk)
    })
    .bind(("khadeemdacosta.ca", 8080))?
    .bind_rustls_0_23(("khadeemdacosta.ca", 8081), tls_config)?
    .run()
    .await
}

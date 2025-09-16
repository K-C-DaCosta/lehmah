use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use oxypress_server::authenticate;
use std::{fs::*, io};

#[get("/")]
async fn hello(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let req_ref = &req;
    authenticate(req_ref, move || async {
        Ok(HttpResponse::Ok().body("hello world"))
    })
    .await
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file =
        io::BufReader::new(File::open("./generated_local_certs/cert.pem").unwrap());
    let mut key_file = io::BufReader::new(File::open("./generated_local_certs/key.pem").unwrap());

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

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("local.khadeemdacosta.ca", 8080))?
    .bind_rustls_0_23(("local.khadeemdacosta.ca", 8081), tls_config)?
    .run()
    .await
}

#[test]
fn query() {
    println!("Hosts info by line");
    std::fs::read_to_string("/etc/hosts")
        .unwrap()
        .lines()
        .filter(|line| line.len() > 2 && !line.starts_with("#"))
        .for_each(|result| {
            println!("line: {:?}", result);
        });

    println!("more indepth parse:");

    std::fs::read_to_string("/etc/hosts")
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
        .flat_map(|line| {
            let mut valid_line_tokens = line
                .split(char::is_whitespace)
                .filter(|token| !token.is_empty());
            let ip = valid_line_tokens.next();
            let hostnames = valid_line_tokens;
            hostnames.filter_map(move |hostname| ip.zip(Some(hostname)))
        })
        .for_each(|result| println!("{:?}", result));
}

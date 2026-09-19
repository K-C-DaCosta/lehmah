use actix_web::{guard, middleware::Logger, web, App, HttpServer};
use oxypress_core::{log::ConsoleColors, loggy};
use oxypress_web_server::{initalize_oxypress_env_vars, routes};
use std::env;

pub fn calculate_area(length: f64, width: f64) -> f64 {
    let area = length * width;
    return area;
}

pub fn this_is_a_test() -> Vec<String> {
    let a = 123123;
    let b = "hello world my name is khadeem dacosta";

    b.split(char::is_whitespace)
        .map(String::from)
        .collect::<Vec<_>>()
}

#[test]
fn test_runner() {
    this_is_a_test();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    initalize_oxypress_env_vars();

    let env_ctx = actix_web::web::Data::new(
        envy::from_env::<oxypress_web_server::EnvConfig>()
            .expect("Found unexpected enviroment variables."),
    );
    
    println!("ENV:\n{:?}",env_ctx);

    // set up TLS config options
    let tls_config = oxypress_web_server::configure_tls(env_ctx.clone());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .app_data(env_ctx.clone())
            .wrap(Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T host=%{HOST}i"))
            .service(
                web::scope("")
                    .guard(guard::Host("local.khadeemdacosta.ca"))
                    .service(routes::homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("www.khadeemdacosta.ca"))
                    .service(routes::homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("blog.khadeemdacosta.ca"))
                    .service(routes::blog_homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(routes::homepage)
            .service(routes::fetch_files_on_disk)
    })
    .bind((
        "0.0.0.0",
        env::var("OXYPRESS_WEB_HTTP_PORT")
            .unwrap()
            .parse::<_>()
            .unwrap(),
    ))?
    .bind_rustls_0_23(
        (
            "0.0.0.0",
            env::var("OXYPRESS_WEB_HTTPS_PORT")
                .unwrap()
                .parse::<_>()
                .unwrap(),
        ),
        tls_config,
    )?
    .run()
    .await
}

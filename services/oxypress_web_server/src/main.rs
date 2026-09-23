use actix_web::{guard, middleware::Logger, web, App, HttpServer};
use oxypress_core::prelude::uuid::Uuid;
use oxypress_web_server::{env_config::EnvConfig, routes};
use sqlx::{
    pool::PoolOptions,
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Connection, Executor, Row,
};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let env_ctx = actix_web::web::Data::new(EnvConfig::get_config_variables());

    println!("ENV:\n{:?}", env_ctx);

    // set up TLS config options
    let tls_config = oxypress_web_server::configure_tls(env_ctx.clone());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = PgPoolOptions::new()
        .connect(&env_ctx.oxypress_database_url)
        .await
        .expect("failed to connect to DB");

    let results = sqlx::query!("Select * FROM users")
        .fetch_all(&pool)
        .await
        .expect("Failed to run select query");

    HttpServer::new(move || {
        App::new()
            .app_data(env_ctx.clone())
            .wrap(Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T host=%{HOST}i",
            ))
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

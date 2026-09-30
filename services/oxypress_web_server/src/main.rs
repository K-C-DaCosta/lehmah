use actix_web::{App, HttpServer, guard, middleware::Logger, web};
use oxypress_core::prelude::uuid::Uuid;
use oxypress_web_server::{env_config::EnvConfig, routes};
use sqlx::{
    ConnectOptions, Connection, Executor, Row,
    pool::PoolOptions,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to initalize rusttls");

    let env_ctx = actix_web::web::Data::new(EnvConfig::get_config_variables());
    let oxypress_server_http_port = env_ctx.oxypress_web_http_port;
    let oxypress_server_https_port = env_ctx.oxypress_web_https_port;
    let oxypress_server_hostname = env_ctx.oxypress_web_hostname.clone();

    println!("ENV:\n{:?}", env_ctx);

    // set up TLS config options
    let tls_config = oxypress_web_server::configure_tls(env_ctx.clone());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = actix_web::web::Data::new(
        PgPoolOptions::new()
            .connect(&env_ctx.oxypress_web_database_url)
            .await
            .expect("failed to connect to DB"),
    );

    let results = sqlx::query!("Select * FROM users")
        .fetch_all(pool.as_ref())
        .await
        .expect("Failed to run select query");
    // Dump vec of results into STDOUT just to check
    for result in results {
        println!("{:?}", result);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(env_ctx.clone())
            .wrap(Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T host=%{HOST}i",
            ))
            .service(
                web::scope("")
                    .guard(guard::Host(format!("local.{oxypress_server_hostname}")))
                    .service(routes::homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host(format!("www.{oxypress_server_hostname}")))
                    .service(routes::homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host(format!("blog.{oxypress_server_hostname}")))
                    .service(routes::blog_homepage)
                    .service(routes::fetch_files_on_disk),
            )
            .service(routes::homepage)
            .service(routes::fetch_files_on_disk)
    })
    .bind(("0.0.0.0", oxypress_server_http_port))?
    .bind_rustls_0_23(("0.0.0.0", oxypress_server_https_port), tls_config)?
    .run()
    .await
}

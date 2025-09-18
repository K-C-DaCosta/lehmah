use actix_web::{App, HttpServer, guard, middleware::Logger, web};
use oxypress_core::{log::ConsoleColors, loggy};
use oxypress_server::initalize_oxypress_env_vars;
use std::env;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    
    initalize_oxypress_env_vars();

    let cwd = std::env::current_dir()?;
    std::env::set_current_dir(cwd.join("oxypress_server"))?;

    loggy!(
        ConsoleColors::YELLOW,
        "Servers CWD set to: {:?}",
        std::env::current_dir().unwrap(),
    );

    // set up TLS config options
    let tls_config = oxypress_server::configure_tls();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            // .wrap(Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T host=%{HOST}i"))
            .service(
                web::scope("")
                    .guard(guard::Host("local.khadeemdacosta.ca"))
                    .service(oxypress_server::homepage)
                    .service(oxypress_server::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("www.khadeemdacosta.ca"))
                    .service(oxypress_server::homepage)
                    .service(oxypress_server::fetch_files_on_disk),
            )
            .service(
                web::scope("")
                    .guard(guard::Host("blog.khadeemdacosta.ca"))
                    .service(oxypress_server::blog_homepage)
                    .service(oxypress_server::fetch_files_on_disk),
            )
            .service(oxypress_server::homepage)
            .service(oxypress_server::fetch_files_on_disk)
    })
    .bind((
        "khadeemdacosta.ca",
        env::var("OXYPRESS_WEB_HTTP_PORT")
            .unwrap()
            .parse::<_>()
            .unwrap(),
    ))?
    .bind_rustls_0_23(
        (
            "khadeemdacosta.ca",
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

use actix_files::NamedFile;
use actix_web::{HttpRequest, error, get, web};
use oxypress_core::{log::*, loggy};

use super::{EnvConfig, translate_to_physical};

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
async fn homepage(env_ctx: web::Data<EnvConfig>, req: HttpRequest) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    if let Some(val) = req.headers().get("host") {
        let host_value = val.to_str().unwrap();
        loggy!(ConsoleColors::YELLOW, "host header = {}", host_value);
    }
    loggy!(ConsoleColors::YELLOW, "homepage requested!");
    let path = translate_to_physical(env_ctx, "static/index.html");
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
}

#[get("/")]
async fn blog_homepage(
    req: HttpRequest,
    env_ctx: web::Data<EnvConfig>,
) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    loggy!(ConsoleColors::YELLOW, "BLOG homepage requested!");
    let path = translate_to_physical(env_ctx, "blog_homepage.html");
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
}

#[get("/{filename:.+\\.[a-zA-z]+}")]
async fn fetch_files_on_disk(
    req: HttpRequest,
    filename: web::Path<String>,
    env_ctx: web::Data<EnvConfig>,
) -> actix_web::Result<NamedFile> {
    let req_ref = &req;
    loggy!(ConsoleColors::YELLOW, "file '{:?}' ", filename);
    let path = translate_to_physical(env_ctx, filename.as_str());
    loggy!(ConsoleColors::YELLOW, "path = {:?} requested!", path);
    authenticate(req_ref, move |_ctx| async {
        NamedFile::open(path).map_err(|e| error::ErrorNotFound(format!("{:?}", e)))
    })
    .await
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

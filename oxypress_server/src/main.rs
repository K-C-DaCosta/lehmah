use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use oxypress_server::authenticate;


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
    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("local.khadeemdacosta.ca", 80))?
    .run()
    .await
}

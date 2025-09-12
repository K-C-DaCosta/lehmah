use actix_web::{HttpRequest, HttpResponse};

pub fn authenticate<Cb, CbOut>(_req: &HttpRequest, callback: Cb) -> CbOut
where
    Cb: Fn() -> CbOut,
    CbOut: Future<Output = actix_web::Result<HttpResponse>>,
{
    // Gets cookie info from request, finds client session from request
    callback() // callback does task now with user state in mind
}



#[test]
fn chrono_test(){

    print!("Chrono = {}",123);

}
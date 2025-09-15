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

#[test]
fn backtrace_check(){

    fn recursive_thing(depth:i32){
        if depth <= 0 {
            let bt = backtrace::Backtrace::new();
            println!("{:?}",bt);
            return; 
        }
        recursive_thing(depth-1);
    }
    recursive_thing(10);
}
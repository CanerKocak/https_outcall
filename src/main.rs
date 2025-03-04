use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
    println!("Starting server on 0.0.0.0:8080");
    
    // Create a TCP listener with SO_REUSEADDR option
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    listener.set_nonblocking(true)?;
    
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .listen(listener)?
    // .bind(("0.0.0.0", 8080))? // Bind to all IPv4 addresses
    // .bind(("::", 8080))? // Bind to all IPv6 addresses - commented out for testing
    .run()
    .await
}
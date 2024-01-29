use actix_files::Files;
use actix_web::{ App, HttpServer, get, Responder};
use actix_files::NamedFile;

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(Files::new("/", "static"))

    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}

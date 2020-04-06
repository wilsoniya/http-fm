use std::path::PathBuf;

use actix_web::{
    App,
    Error,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    web,
};
use futures::future::{ready, Ready};

mod errors;
mod fs;

impl Responder for fs::DirectoryListing {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let response = HttpResponse::Ok().json(self);
        ready(Ok(response))
    }
}

async fn index() -> impl Responder {
    "Hello world!"
}

async fn list_directory(params: web::Path<(PathBuf,)>) -> impl Responder {
    let path = &params.0;
    fs::ls(path)
}

#[actix_rt::main]
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ls/{fpath:.*}", web::get().to(list_directory))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

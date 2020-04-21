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
use async_std;
use async_std::io::ReadExt;
use async_std::prelude::StreamExt;

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

async fn fetch(params: web::Path<(String, PathBuf)>) -> impl Responder {
    let (id, path) = params.into_inner();
    let file = async_std::fs::File::open("/home/wilsoniya/devel/http_fm/Cargo.lock").await.unwrap();
    let bytes = file
        .bytes()
        .map(|maybe_bytes| {
            let mut buf = Vec::<u8>::with_capacity(1024);
            maybe_bytes.map(|byte| {
                web::Bytes::from(vec![byte])
            })
        });
    HttpResponse::Ok()
        .streaming(bytes)
}


#[actix_rt::main]
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ls/{fpath:.*}", web::get().to(list_directory))
            .route("/fetch/{id}/{fpath:.*}", web::get().to(fetch))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

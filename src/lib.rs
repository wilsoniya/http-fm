use std::path::PathBuf;
use std::path::Path;

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
use futures::stream::StreamExt;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

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
    let (_id, path) = params.into_inner();
    let abs_path = Path::new("/").join(path);

    File::open(abs_path).await.map(|file| {
        let stream = FramedRead::new(file, BytesCodec::new())
            .map(|maybe_bytesmut| maybe_bytesmut.map(|bytesmut| bytesmut.into()));

        HttpResponse::Ok()
            .streaming(stream)
    })
}


#[actix_rt::main]
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ls/{fpath:.*}", web::get().to(list_directory))
            .route("/fetch/{fpath:.*}", web::get().to(fetch))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

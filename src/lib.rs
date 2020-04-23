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
use bytes::Bytes;
use futures::future::{ready, Ready};
use futures::stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};

mod errors;
mod fs;

impl Responder for fs::DirectoryListing {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let response = HttpResponse::Ok()
            .json(self);
        ready(Ok(response))
    }
}

impl Responder for fs::FSItem {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let response = match self {
            Self::Directory(directory_listing) => {
                HttpResponse::Ok()
                    .json(directory_listing)
            },
            Self::File(file, file_length) => {
                let stream = FramedRead::new(file, BytesCodec::new())
                    .map(|maybe_bytesmut| maybe_bytesmut.map(Bytes::from));
                HttpResponse::Ok()
                    .content_length(file_length)
                    .streaming(stream)
            },
        };

        ready(Ok(response))
    }
}

async fn index() -> impl Responder {
    "Hello world!"
}

async fn share(params: web::Path<(String, PathBuf)>) -> impl Responder {
    let (_id, path) = params.into_inner();
    let abs_path = Path::new("/").join(path);
    fs::FSItem::new(&abs_path).await
}

#[actix_rt::main]
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/share/{id}/{fpath:.*}", web::get().to(share))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

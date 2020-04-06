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

async fn list_directory() -> impl Responder {
    fs::DirectoryListing {
        items: vec![
            fs::FSItem::Directory {
                path: String::from("/path/to/fart"),
            },
            fs::FSItem::File {
                path: String::from("/path/to/w33d.txt"),
                size_bytes: 420,
            },
        ]
    }
}

#[actix_rt::main]
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ls", web::get().to(list_directory))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

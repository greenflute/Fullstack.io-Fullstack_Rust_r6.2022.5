use actix_web::{middleware, server, App, HttpRequest, Json, Result};
use serde::Serialize;

#[derive(Serialize)]
struct IndexResponse {
    message: String,
}

fn index(req: &HttpRequest) -> Result<Json<IndexResponse>> {
    let headers = req.headers();
    let hello = headers
        .get("hello")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_else(|| "world");
    Ok(Json(IndexResponse {
        message: hello.to_owned(),
    }))
}

pub struct MessageApp {
    port: u16,
}

impl MessageApp {
    pub fn new(port: u16) -> Self {
        MessageApp { port }
    }

    pub fn run(&self) {
        let sys = actix::System::new("messages-actix");
        server::new(move || {
            App::new()
                .middleware(middleware::Logger::default())
                .resource("/", |r| r.f(index))
        })
        .bind(("127.0.0.1", self.port))
        .unwrap()
        .start();
        println!("Started http server: 127.0.0.1:{}", self.port);
        let _ = sys.run();
    }
}

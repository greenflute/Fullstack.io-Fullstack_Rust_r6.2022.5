use actix_web::{
    error::{Error, InternalError, JsonPayloadError},
    http, middleware, server, App, HttpRequest, HttpResponse, Json, Result, State,
};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::{Arc, Mutex};

const LOG_FORMAT: &'static str = r#""%r" %s %b "%{User-Agent}i" %D"#;
static SERVER_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

struct AppState {
    server_id: usize,
    request_count: Cell<usize>,
    messages: Arc<Mutex<Vec<String>>>,
}

#[derive(Serialize)]
struct IndexResponse {
    server_id: usize,
    request_count: usize,
    messages: Vec<String>,
}

#[derive(Deserialize)]
struct PostInput {
    message: String,
}

#[derive(Serialize)]
struct PostResponse {
    server_id: usize,
    request_count: usize,
    message: String,
}

#[derive(Serialize)]
struct PostError {
    server_id: usize,
    request_count: usize,
    error: String,
}

fn index(req: &HttpRequest<AppState>) -> Result<Json<IndexResponse>> {
    let state = req.state();
    let request_count = state.request_count.get() + 1;
    state.request_count.set(request_count);
    let ms = state.messages.lock().unwrap();

    Ok(Json(IndexResponse {
        server_id: state.server_id,
        request_count,
        messages: ms.clone(),
    }))
}

fn post((msg, req): (Json<PostInput>, HttpRequest<AppState>)) -> Result<Json<PostResponse>> {
    let state = req.state();
    let request_count = state.request_count.get() + 1;
    state.request_count.set(request_count);
    let mut ms = state.messages.lock().unwrap();
    ms.push(msg.message.clone());

    Ok(Json(PostResponse {
        server_id: state.server_id,
        request_count,
        message: msg.message.clone(),
    }))
}

fn clear(state: State<AppState>) -> Result<Json<IndexResponse>> {
    let request_count = state.request_count.get() + 1;
    state.request_count.set(request_count);
    let mut ms = state.messages.lock().unwrap();
    ms.clear();

    Ok(Json(IndexResponse {
        server_id: state.server_id,
        request_count,
        messages: vec![],
    }))
}

fn post_error(err: JsonPayloadError, req: &HttpRequest<AppState>) -> Error {
    let state = req.state();
    let request_count = state.request_count.get() + 1;
    state.request_count.set(request_count);
    let post_error = PostError {
        server_id: state.server_id,
        request_count,
        error: format!("{}", err),
    };
    InternalError::from_response(err, HttpResponse::BadRequest().json(post_error)).into()
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
        let messages = Arc::new(Mutex::new(vec![]));
        server::new(move || {
            App::with_state(AppState {
                server_id: SERVER_COUNTER.fetch_add(1, Ordering::SeqCst),
                request_count: Cell::new(0),
                messages: messages.clone(),
            })
            .middleware(middleware::Logger::new(LOG_FORMAT))
            .resource("/", |r| r.f(index))
            .resource("/send", |r| {
                r.method(http::Method::POST).with_config(post, |config| {
                    (config.0).0.error_handler(post_error);
                })
            })
            .resource("/clear", |r| r.method(http::Method::POST).with(clear))
        })
        .bind(("127.0.0.1", self.port))
        .unwrap()
        .start();
        println!("Started http server: 127.0.0.1:{}", self.port);
        let _ = sys.run();
    }
}

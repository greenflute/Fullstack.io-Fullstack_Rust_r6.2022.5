use actix_web::{middleware, server, App, HttpRequest, Json, Result};
use serde::Serialize;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::{Arc, Mutex};

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

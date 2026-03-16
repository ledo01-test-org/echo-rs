use std::{collections::VecDeque, convert::Infallible, net::SocketAddr, sync::Arc, time::Instant};

use tokio::sync::Mutex;

use axum::{
    body::Bytes,
    extract::{OriginalUri, Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::{net::TcpListener, sync::broadcast};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;

mod health;

const DASHBOARD_HTML: &str = include_str!("dashboard.html");

#[derive(Clone, Serialize)]
struct CapturedRequest {
    id: String,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timestamp: DateTime<Utc>,
}

const MAX_HISTORY: usize = 100;

struct AppState {
    tx: broadcast::Sender<CapturedRequest>,
    history: Mutex<VecDeque<CapturedRequest>>,
}

async fn dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn favicon() -> impl IntoResponse {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y=".9em" font-size="90">📡</text></svg>"#;
    ([(header::CONTENT_TYPE, "image/svg+xml")], svg)
}

async fn history_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let history = state.history.lock().await;
    let requests: Vec<_> = history.iter().cloned().collect();
    axum::Json(requests)
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        result
            .ok()
            .map(|req| Ok(Event::default().data(serde_json::to_string(&req).unwrap_or_default())))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn capture(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let headers_vec: Vec<(String, String)> = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    let body_text = String::from_utf8(body.to_vec()).ok();

    let req = CapturedRequest {
        id: uuid::Uuid::new_v4().to_string(),
        method: method.to_string(),
        path: uri.to_string(),
        headers: headers_vec,
        body: body_text,
        timestamp: Utc::now(),
    };

    {
        let mut history = state.history.lock().await;
        history.push_front(req.clone());
        if history.len() > MAX_HISTORY {
            history.pop_back();
        }
    }

    let _ = state.tx.send(req);

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], "ok")
}

async fn access_log(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    info!(
        "{} {} {} {:?}",
        method,
        uri,
        response.status().as_u16(),
        start.elapsed()
    );

    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let (tx, _) = broadcast::channel::<CapturedRequest>(100);
    let state = Arc::new(AppState {
        tx,
        history: Mutex::new(VecDeque::new()),
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let app = Router::new()
        .route("/_dashboard", get(dashboard))
        .route("/_history", get(history_handler))
        .route("/_sse", get(sse_handler))
        .route("/favicon.ico", get(favicon))
        .route("/_health", get(health::health_check))
        .route("/_ready", get(health::readiness_check))
        .fallback(capture)
        .layer(axum::middleware::from_fn(access_log))
        .with_state(state);

    info!("Echo server started on http://{}", addr);
    info!("Dashboard available at http://{}/_dashboard", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

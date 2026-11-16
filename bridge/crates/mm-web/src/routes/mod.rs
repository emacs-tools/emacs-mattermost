use crate::{state::WebState, templates};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use mm_core::BridgeStatus;
use serde_json::{json, Value};

pub fn router(state: WebState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/status", get(status))
        .with_state(state)
}

async fn index(State(state): State<WebState>) -> Html<String> {
    Html(templates::index_html(&state.status))
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "state": "running"
    }))
}

async fn status(State(state): State<WebState>) -> Json<BridgeStatus> {
    Json(state.status)
}

#[allow(dead_code)]
fn _value_type_check(value: Value) -> Value {
    value
}

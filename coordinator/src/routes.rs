use crate::{handlers::{build_task, build_task_response, metrics_handler}, SharedState};
use axum::{routing::get, routing::post, Extension, Router};
use std::sync::Arc;

pub fn create_routes(shared_state: SharedState) -> Router {
    Router::new()
        .route("/build_task", post(build_task))
        .route("/build_task_response", post(build_task_response))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(shared_state))
}

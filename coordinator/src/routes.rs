use crate::handlers::{build_task, build_task_response, metrics_handler};
use crate::SharedState;
use axum::{routing::get, routing::post, Extension, Router};
use etcd_client::Client as eClient;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn create_routes(shared_state: SharedState, etcd_client: Arc<Mutex<eClient>>) -> Router {
    Router::new()
        .route("/build_task", post(build_task))
        .route("/build_task_response", post(build_task_response))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(shared_state))
        .layer(Extension(etcd_client))
}

pub fn create_routes2(shared_state: SharedState) -> Router {
    Router::new()
        .route("/build_task", post(build_task))
        .route("/build_task_response", post(build_task_response))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(shared_state))
}

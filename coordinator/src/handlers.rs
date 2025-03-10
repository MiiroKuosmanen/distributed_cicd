use std::any::Any;

use crate::discover_service;
use crate::models::{Task, TaskResult};
use crate::AppState;
use crate::SharedState;
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::post,
    Router,
};
use etcd_client::Client as eClient;
use futures::Future;
use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, Registry, TextEncoder};
use redis::Commands;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
// Define global counters
lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: IntCounter =
        IntCounter::new("http_requests_total", "Total HTTP requests received").unwrap();
    static ref BUILD_TASKS_TOTAL: IntCounter =
        IntCounter::new("build_tasks_total", "Total build tasks processed").unwrap();
    static ref REGISTRY: Registry = {
        let reg = Registry::new();
        reg.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
        reg.register(Box::new(BUILD_TASKS_TOTAL.clone())).unwrap();
        reg
    };
}
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
//Extension(etcd_client): Extension<Arc<tokio::sync::Mutex<eClient>>>,
#[axum_macros::debug_handler]
pub async fn build_task(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<Task>,
) -> impl IntoResponse {
    HTTP_REQUESTS_TOTAL.inc();
    BUILD_TASKS_TOTAL.inc();

    // Discover the worker address
    //let worker_address = discover_service(etcd_client.clone(), "worker-service").await;
    let worker_address = Some("worker-service.cicd.svc.cluster.local:5001".to_string());

    if let Some(worker_url) = worker_address {
        let mut clock = state.clock.lock().await;
        clock.increment();
        println!("Logical time is now: {}", clock.get_time());

        let client = Client::new();
        let response = client
            .post(format!("http://{}/execute_task", worker_url))
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    println!("Task submitted successfully to {}", worker_url);
                    let result: TaskResult = res.json().await.unwrap();
                    (StatusCode::OK, Json(result)).into_response()
                } else {
                    (
                        StatusCode::BAD_GATEWAY,
                        format!("Worker failed with status: {}", res.status()),
                    )
                        .into_response()
                }
            }
            Err(err) => (
                StatusCode::BAD_GATEWAY,
                format!("Failed to connect to worker: {}", err),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Worker service not found in service discovery",
        )
            .into_response()
    }
}

pub async fn build_task2(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<Task>,
) -> impl IntoResponse {
    let workers = vec![
        "http://worker1:5001/execute_task", // Worker 1
        "http://worker2:5002/execute_task", // Worker 2
    ];
    let mut round_robin_state = state.state.lock().await;
    let worker_url = workers[(*round_robin_state % workers.len() as u8) as usize];

    // Switch to the next worker for the next task
    *round_robin_state = (*round_robin_state + 1) % workers.len() as u8;
    let mut clock = state.clock.lock().await;
    clock.increment();
    println!("Logical time is now: {}", clock.get_time());
    // Send the task to the worker
    let client = Client::new();
    //println!("json: {:?}", &payload);
    let task = Task {
        id: 1,
        repository: "Build this project".to_string(),
        task: "code-review".to_string(),
    };
    println!("test1");
    let response = client.post(worker_url).json(&task).send().await;
    //println!("Response: {:?}", response.);
    // Handle worker response
    println!("test2");
    match response {
        Ok(res) => {
            if res.status().is_success() {
                // Parse worker's response
                println!("Task submitted successfully: {:?}", worker_url);
                let result: TaskResult = res.json().await.unwrap();
                (StatusCode::OK, Json(result)).into_response()
            } else {
                println!("Response: {:?}", res);
                (
                    StatusCode::BAD_GATEWAY,
                    format!("Worker failed with status: {}", res.status()),
                )
                    .into_response()
            }
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            format!("Failed to connect to worker: {}", err),
        )
            .into_response(),
    }
}

pub async fn build_task_response(Json(payload): Json<Task>) -> impl IntoResponse {
    let serialized_task = serde_json::to_string(&payload).unwrap();

    //con.lpush("task_queue", serialized_task).unwrap();

    println!("Task finished: {:?}", serialized_task);

    /*match result {
        Ok(_) => (StatusCode::CREATED, "build task created successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create build task").into_response(),
    }*/
    (StatusCode::CREATED, "build task finished successfully").into_response()
}

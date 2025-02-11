use axum::{
    extract::Json,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tokio::net::TcpListener;
use prometheus::{Encoder, TextEncoder, Registry, IntCounter};
use lazy_static::lazy_static;

lazy_static! {
    static ref TASKS_PROCESSED_TOTAL: IntCounter =
        IntCounter::new("tasks_processed_total", "Total tasks processed").unwrap();
    static ref REGISTRY: Registry = {
        let reg = Registry::new();
        reg.register(Box::new(TASKS_PROCESSED_TOTAL.clone())).unwrap();
        reg
    };
}


#[derive(Debug, Deserialize)]
struct Task {
    id: u32,
    repository: String,
    branch: String,
}

#[derive(Debug, Serialize)]
struct TaskResult {
    id: u32,
    status: String,
    result: String,
}
async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}


#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("WORKER_PORT").unwrap_or("5001".to_string()).parse().unwrap();

    let app = Router::new()
        .route("/execute_task", post(handle_task))
        .route("/health", get(health_check)) // ✅ Add health check
        .route("/metrics", get(metrics_handler));

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Worker running on {}", addr);

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn handle_task(Json(task): Json<Task>) -> Json<TaskResult> {
    //HTTP_REQUESTS_TOTAL.inc(); // ✅ Count each request
    println!("Received task: {:?}", task);
    let mut x: i128 = 0;
    for i in 0_..1_000_000_000 {
        x = x + i;
        //println!("hello");
    }
    // Simulate task processing
    //sleep(Duration::from_secs(3)).await;

    //TASKS_PROCESSED_TOTAL.inc(); // ✅ Count processed tasks
    let result = format!("Processed payload: {}", task.id);
    println!("Task {} completed with result: {}", task.id, result);

    // Return result to the coordinator
    Json(TaskResult {
        id: task.id,
        status: "completed".to_string(),
        result,
    })
}

// ✅ Health Check Endpoint for Kubernetes
async fn health_check() -> &'static str {
    "OK"
}

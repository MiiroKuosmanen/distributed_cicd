use axum::{
    extract::Json,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("WORKER_PORT").unwrap_or("5001".to_string()).parse().unwrap();

    let app = Router::new()
        .route("/execute_task", post(handle_task))
        .route("/health", get(health_check)); // ✅ Add health check

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Worker running on {}", addr);

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn handle_task(Json(task): Json<Task>) -> Json<TaskResult> {
    println!("Received task: {:?}", task);

    // Simulate task processing
    sleep(Duration::from_secs(3)).await;

    // Send result back to the coordinator
    let result = format!("Processed payload: {}", task.id);
    println!("Task {} completed with result: {}", task.id, result);

    // Return result to the coordinator
    Json(TaskResult {
        id: task.id,
        status: "completed".to_string(),
        result,
    })
}

// ✅ New Health Check Endpoint for Kubernetes
async fn health_check() -> &'static str {
    "OK"
}

use axum::{
    extract::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr};
use tokio::time::{sleep, Duration};
use axum::serve::WithGracefulShutdown;
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
    // Define the Axum app
    let app = Router::new().route("/execute_task", post(handle_task));

    // Run the worker server
    let listener = TcpListener::bind("0.0.0.0:5002").await.unwrap();
    axum::serve(listener, app.into_make_service())
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
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
        result: result,
    })
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = signal(SignalKind::terminate()).expect("failed to install signal handler");
    stream.recv().await;
}

use std::any::Any;

use crate::models::{Task, TaskResult};
use crate::SharedState;
use axum::http::StatusCode;
use axum::{extract::Path, response::IntoResponse, Extension, Json};
use redis::Commands;
use reqwest::Client;


pub async fn build_task(Extension(state): Extension<SharedState>, Json(payload): Json<Task>,) -> impl IntoResponse {
    let workers = vec![
        "http://127.0.0.1:5001/execute_task", // Worker 1
        "http://127.0.0.1:5002/execute_task", // Worker 2
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
    println!("json: {:?}", &payload);
    let task = Task {
        id: 1,
        repository: "Build this project".to_string(),
        branch: "test".to_string()
    };
    let response = client.post(worker_url).json(&task).send().await;
    //println!("Response: {:?}", test);
    // Handle worker response
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

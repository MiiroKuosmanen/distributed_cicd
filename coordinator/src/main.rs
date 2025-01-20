
mod routes;
mod handlers;
mod models;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::serve::WithGracefulShutdown;
use tokio::net::TcpListener;
use crate::routes::create_routes;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub struct LogicalClock {
    pub time: u64,
}

impl LogicalClock {
    pub fn new() -> Self {
        Self { time: 0 }
    }

    pub fn increment(&mut self) {
        self.time += 1;
    }

    pub fn get_time(&self) -> u64 {
        self.time
    }
}

#[derive(Debug)]
pub struct AppState {
    pub state: Mutex<u8>, // Round-robin state: 1 or 2 for Worker 1 or Worker 2
    pub clock: Mutex<LogicalClock>, // Logical clock for time coordination
}
pub type SharedState = Arc<AppState>;
#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        state: Mutex::new(1), // Start round-robin at worker 1
        clock: Mutex::new(LogicalClock::new()),
    });
    let app = create_routes(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = signal(SignalKind::terminate()).expect("failed to install signal handler");
    stream.recv().await;
}

mod handlers;
mod models;
mod routes;
use crate::routes::{create_routes, create_routes2};
use axum::serve::WithGracefulShutdown;
use etcd_client::{Client as eClient, GetOptions, PutOptions};
use prometheus::{Encoder, Registry, TextEncoder};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use futures::stream::StreamExt; // Import StreamExt for async iteration

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
    //let etcd_client = init_etcd_client().await;
    //register_worker_service(etcd_client.clone()).await;
    //let app = create_routes(shared_state.clone(), etcd_client.clone());
    /*let node_ip = "192.168.49.2"; // replace with the actual node IP
    let node_port = 32000; // your node port configuration
    register_service(
        etcd_client.clone(),
        "coordinator",
        &format!("{}:{}", node_ip, node_port),
    )
    .await;*/
    //let service_address = "coordinator.cicd.svc.cluster.local:3000";
    //let node_ip = std::env::var("POD_IP").unwrap_or("127.0.0.1".to_string());
    //let service_address = format!("{}:3000", node_ip);
    /*
    register_service(etcd_client.clone(), "coordinator", &service_address).await;
    if elect_leader(etcd_client).await {
        println!("This node is the leader!");
    } else {
        println!("This node is a follower!");
    }*/
    let app = create_routes2(shared_state.clone());
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn init_etcd_client() -> Arc<Mutex<eClient>> {
    let client = eClient::connect(["http://etcd-service:2379"], None)
        .await
        .unwrap();
    Arc::new(Mutex::new(client))
}

async fn elect_leader(cli: Arc<Mutex<eClient>>) -> bool {
    let mut cli_locked = cli.lock().await;

    // Create a lease with a 5-second duration
    let lease = cli_locked.lease_grant(5, None).await.unwrap();

    // Set the lease using PutOptions and wrap it in Some
    let options = Some(PutOptions::new().with_lease(lease.id()));

    // Use the lease to try and set a key as the leader
    let put_resp = cli_locked.put("leader-key", "instance-id", options).await;

    if put_resp.is_ok() {
        println!("Leadership acquired.");
        let cli_clone = Arc::clone(&cli);
        tokio::spawn(async move {
            loop {
                let mut cli_clone_locked = cli_clone.lock().await;
                // Keep the lease alive
                match cli_clone_locked.lease_keep_alive(lease.id()).await {
                    Ok(_) => println!("Lease kept alive"),
                    Err(e) => println!("Failed to keep lease alive: {:?}", e),
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
            }
        });
        return true;
    }

    println!("Failed to acquire leadership.");
    false
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = signal(SignalKind::terminate()).expect("failed to install signal handler");
    stream.recv().await;
}
/*
async fn register_service(cli: Arc<Mutex<eClient>>, service_name: &str, service_address: &str) {
    let mut cli_locked = cli.lock().await;
    cli_locked
        .put(format!("/services/{}", service_name), service_address, None)
        .await
        .unwrap();
    println!(
        "Service registered with etcd: {} -> {}",
        service_name, service_address
    );
}*/
async fn register_service(cli: Arc<Mutex<eClient>>, service_name: &str, service_address: &str) {
    let mut client = cli.lock().await;
    match client
        .put(format!("/services/{}", service_name), service_address, None)
        .await
    {
        Ok(_) => println!(
            "Service registered: {} -> {}",
            service_name, service_address
        ),
        Err(e) => eprintln!(
            "Failed to register service: {} -> {}. Error: {:?}",
            service_name, service_address, e
        ),
    }
}

async fn register_worker_service(cli: Arc<Mutex<eClient>>) {
    // Service address using Kubernetes DNS: service-name.namespace.svc.cluster.local
    let service_address = "worker-service.cicd.svc.cluster.local:5001";
    let mut client = cli.lock().await;
    match client
        .put("/services/worker-service", service_address, None)
        .await
    {
        Ok(_) => println!("Registered worker service at {}", service_address),
        Err(e) => eprintln!("Failed to register worker service: {:?}", e),
    }
}
async fn discover_service(cli: Arc<Mutex<eClient>>, service_name: &str) -> Option<String> {
    let mut cli_locked = cli.lock().await;
    let response = cli_locked
        .get(format!("/services/{}", service_name), None)
        .await
        .unwrap();
    if let Some(kv) = response.kvs().first() {
        return Some(kv.value_str().unwrap().to_string());
    }
    None
}

async fn watch_key(cli: Arc<Mutex<eClient>>, key: &str) {
    let mut cli_locked = cli.lock().await;
    let (_watcher, mut stream) = cli_locked.watch(key, None).await.unwrap();

    while let Some(Ok(watch_event)) = stream.next().await {
        for event in watch_event.events() {
            println!(
                "Key change: {:?} - {:?}",
                event.event_type(),
                event.kv().unwrap().value_str()
            );
            // Update local state or perform actions based on key change
        }
    }
}

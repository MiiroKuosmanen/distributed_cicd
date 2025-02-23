use dagger_sdk::{DaggerConn, File};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dagger_sdk::connect(|conn| async move {
        println!("🚀 Running Rust Build Pipeline...");
        let build = build_rust_binary(&conn).await?;
        let file_id = build.id().await?;
        println!("✅ Build complete! File ID: {:?}", file_id);
        Ok(())
    })
    .await
    .map_err(|e| eyre::Report::new(e))?; // Convert ConnectError to eyre::Report

    Ok(())
}

// Rust Build Pipeline - Only "cargo build --release"
async fn build_rust_binary(conn: &DaggerConn) -> Result<File> {
    let client_project = conn
        .host()
        .directory("/home/maso77/repos/distributed_cicd/client");
    let shared_project = conn
        .host()
        .directory("/home/maso77/repos/distributed_cicd/shared"); // Mount shared

    let file = conn
        .container()
        .from("rust:1.77.2-alpine3.19")
        .with_exec(vec!["apk", "add", "build-base", "musl"])
        .with_mounted_directory("/app", client_project)
        .with_mounted_directory("/shared", shared_project) // Mount shared
        .with_workdir("/app")
        .with_exec(vec!["cargo", "build", "--release"])
        .file("./target/release/client");

    Ok(file)
}

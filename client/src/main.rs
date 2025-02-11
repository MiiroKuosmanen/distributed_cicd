use clap::Parser;
use reqwest::Client;
use serde::Serialize;

/// Command-line arguments parser using Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Description of the task to submit
    #[arg(short, long)]
    task: String,
}

#[derive(Serialize)]
struct Task {
    id: u32,
    repository: String,
    branch: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Parse CLI arguments
    let args = Args::parse();

    // Task data to be sent
    let task = Task {
        id: 1, // This can later be replaced with a generated ID or UUID
        repository: args.task,
        branch: "test".to_string(),
    };

    // Create an HTTP client
    let client = Client::new();

    // Send the task to the Coordinator
    let response = client
        .post("http://192.168.49.2:32000/build_task")
        .json(&task)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Task finished successfully: {}", response.text().await?);
    } else {
        eprintln!(
            "Failed to submit task. Status: {}",
            response.status()
        );
    }

    Ok(())
}

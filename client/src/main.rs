use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde::Serialize;
use serde_json::{self, Value};
use thiserror::Error;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Description of the task to submit
    #[arg(short, long, value_enum)]
    task: TaskType,

    /// Path to the repository (if required)
    #[arg(short, long, default_value = "")]
    repo_path: String,
}
#[derive(Error, Debug)]
enum CustomError {
    #[error("Request error: {0}")]
    Reqwest(reqwest::Error),

    #[error("JSON error: {0}")]
    SerdeJson(serde_json::Error),
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> CustomError {
        CustomError::Reqwest(err)
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(err: serde_json::Error) -> CustomError {
        CustomError::SerdeJson(err)
    }
}

#[derive(ValueEnum, Clone, Debug)]
enum TaskType {
    Build,
    Test,
    Lint,
    Review,
}

#[derive(Serialize)]
struct Task {
    id: u32,
    task: String,
    repository: String,
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    // Parse CLI arguments
    let args = Args::parse();

    // Map TaskType to task type strings
    let task_type_str = match args.task {
        TaskType::Build => "rust-build",
        TaskType::Test => "rust-test",
        TaskType::Lint => "python-lint2",
        TaskType::Review => "code-review",
    };

    // Task data to be sent
    let task = Task {
        id: 1, // This can later be replaced with a generated ID or UUID
        task: task_type_str.to_owned(),
        repository: "/python/app.py".to_string(),
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
        let response_text = response.text().await?;

        // Deserialize the response text into a Value
        let response_json: Value = serde_json::from_str(&response_text)?;

        // Pretty print the JSON response
        let pretty_response = serde_json::to_string_pretty(&response_json)?;

        println!("Task finished successfully: {}", pretty_response);
    } else {
        eprintln!("Failed to submit task. Status: {}", response.status());
    }

    Ok(())
}

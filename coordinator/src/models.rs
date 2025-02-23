use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: u32,
    pub repository: String,
    pub task: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskResult {
    id: u32,
    status: String,
    result: String,
}

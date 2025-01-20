use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: u32,
    pub repository: String,
    pub branch: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TaskResult {
    id: u32,
    status: String,
    result: String,
}

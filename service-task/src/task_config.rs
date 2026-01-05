use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TaskConfig {
    pub max_running_tasks:usize,
}
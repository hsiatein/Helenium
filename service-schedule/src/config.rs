use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScheduleConfig {
    pub schedule_dir: String,
    pub offset: i32,
}

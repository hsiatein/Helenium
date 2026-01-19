use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScheduleConfig {
    pub offset: i32,
}

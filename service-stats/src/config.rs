use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct StatsConfig {
    pub duration: usize,
}

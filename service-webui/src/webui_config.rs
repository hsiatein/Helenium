use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WebuiConfig {
    pub port: String,
    pub session_buffer: usize,
}

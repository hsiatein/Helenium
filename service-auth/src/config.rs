use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthConfig {
    pub pub_keys: Vec<String>,
}

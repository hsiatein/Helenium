use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WebuiConfig{
    pub port:String,
}
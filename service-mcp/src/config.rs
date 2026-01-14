use std::collections::HashMap;
use heleny_proto::HelenyProcessCommand;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Config {
    pub mcp_servers:HashMap<String,HelenyProcessCommand>
}
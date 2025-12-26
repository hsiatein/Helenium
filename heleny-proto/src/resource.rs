use std::collections::VecDeque;

use crate::health::KernelHealth;

#[derive(Debug,Clone)]
pub struct Resource{
    pub name:String,
    pub payload:ResourcePayload
}

impl Resource {
    pub fn new(name:&str,payload:ResourcePayload)->Self{
        Self{name:name.to_string(),payload}
    }
}

pub static TOTAL_BUS_TRAFFIC:&'static str="TotalBusTraffic";

#[derive(Debug,Clone)]
pub enum ResourcePayload {
    Health(KernelHealth),
    TotolBusTraffic(VecDeque<usize>),
}

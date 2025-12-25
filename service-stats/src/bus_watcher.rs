use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::warn;

/// 主要职责是监控Bus的流量进行统计
pub struct BusWatcher {
    _handle: JoinHandle<Result<()>>,
    total_traffic: Arc<Mutex<VecDeque<usize>>>,
}

impl BusWatcher {
    pub fn new(
        duration: usize,
        mut bus_rx: mpsc::Receiver<HashMap<&'static str, usize>>,
    ) -> Result<BusWatcher> {
        let total_traffic = Arc::new(Mutex::new(VecDeque::new()));
        let total_traffic_ = total_traffic.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = bus_rx.recv()=>{
                        if let Err(e) =handle(duration,&total_traffic_,msg).await {
                            warn!("BusWatcher: {}",e)
                        }
                    }
                }
            }
        });
        Ok(Self {
            _handle:handle,
            total_traffic,
        })
    }

    pub fn _abort(&self) {
        self._handle.abort();
    }

    pub fn get_total_traffic(&self) -> Result<VecDeque<usize>> {
        match self.total_traffic.lock() {
            Ok(traffic) => Ok(traffic.to_owned()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

async fn handle(
    duration: usize,
    total_traffic: &Arc<Mutex<VecDeque<usize>>>,
    msg: HashMap<&'static str, usize>,
) -> Result<()> {
    match total_traffic.lock() {
        Ok(mut traffic) => {
            let total = msg.values().cloned().sum();
            traffic.push_back(total);
            if traffic.len() > duration {
                traffic.pop_front();
            }
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

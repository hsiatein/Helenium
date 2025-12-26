use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use heleny_proto::resource::ResourcePayload;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::warn;

/// 主要职责是监控Bus的流量进行统计
pub struct BusWatcher {
    _handle: JoinHandle<Result<()>>,
    total_traffic: Arc<Mutex<VecDeque<usize>>>,
}

impl BusWatcher {
    pub fn new(
        duration: usize,
        mut bus_rx: mpsc::Receiver<HashMap<String, usize>>,
    ) -> Result<(BusWatcher,watch::Receiver<ResourcePayload>)> {
        let (tx,rx)=watch::channel(ResourcePayload::TotolBusTraffic(VecDeque::new()));
        let total_traffic = Arc::new(Mutex::new(VecDeque::new()));
        let total_traffic_ = total_traffic.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = bus_rx.recv()=>{
                        if let Err(e) =handle(duration,&total_traffic_,msg,&tx).await {
                            warn!("BusWatcher: {}",e)
                        }
                    }
                }
            }
        });
        Ok((Self {
            _handle: handle,
            total_traffic,
        },rx))
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
    msg: HashMap<String, usize>,
    tx:&watch::Sender<ResourcePayload>,
) -> Result<()> {
    match total_traffic.lock() {
        Ok(mut traffic) => {
            let total = msg.values().cloned().sum();
            traffic.push_back(total);
            if traffic.len() > duration {
                traffic.pop_front();
            }
            debug!("{:?}",traffic.to_owned());
            tx.send(ResourcePayload::TotolBusTraffic(traffic.to_owned()))?;
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

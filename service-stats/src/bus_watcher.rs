use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use heleny_proto::ResourcePayload;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;
// use tracing::debug;
use tracing::warn;

/// 主要职责是监控Bus的流量进行统计
pub struct BusWatcherHandle {
    _handle: JoinHandle<Result<()>>,
    total_traffic: Arc<Mutex<VecDeque<(DateTime<Local>, usize)>>>,
}

impl BusWatcherHandle {
    pub fn new(
        duration: usize,
        mut bus_rx: mpsc::Receiver<(String, String)>,
    ) -> Result<(BusWatcherHandle, watch::Receiver<ResourcePayload>)> {
        let (tx, rx) = watch::channel(ResourcePayload::TotalBusTraffic(VecDeque::new()));
        let total_traffic = Arc::new(Mutex::new(VecDeque::new()));
        let total_traffic_ = total_traffic.clone();
        let handle = tokio::spawn(async move {
            let mut bus_watcher = BusWatcher::new(duration, total_traffic_, tx);
            let mut tick_interval = interval(Duration::from_secs(1));
            tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    Some(msg) = bus_rx.recv()=>{
                        if let Err(e) =bus_watcher.handle(msg) {
                            warn!("BusWatcher: {}",e)
                        }
                    }
                    _ = tick_interval.tick() =>{
                        if let Err(e) =bus_watcher.handle_tick() {
                            warn!("BusWatcher: {}",e)
                        }
                    }
                }
            }
        });
        Ok((
            Self {
                _handle: handle,
                total_traffic,
            },
            rx,
        ))
    }

    pub fn _abort(&self) {
        self._handle.abort();
    }

    pub fn get_total_traffic(&self) -> Result<VecDeque<(DateTime<Local>, usize)>> {
        match self.total_traffic.lock() {
            Ok(traffic) => Ok(traffic.to_owned()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

pub struct BusWatcher {
    duration: usize,
    current_time: DateTime<Local>,
    count: usize,
    total_traffic: Arc<Mutex<VecDeque<(DateTime<Local>, usize)>>>,
    tx: watch::Sender<ResourcePayload>,
}

impl BusWatcher {
    pub fn new(
        duration: usize,
        total_traffic: Arc<Mutex<VecDeque<(DateTime<Local>, usize)>>>,
        tx: watch::Sender<ResourcePayload>,
    ) -> Self {
        if let Ok(mut traffic) = total_traffic.lock() {
            let fill = duration - traffic.len();
            for i in 1..fill + 1 {
                traffic.push_front((Local::now() - chrono::Duration::seconds(i as i64), 0));
            }
        };
        Self {
            duration,
            current_time: Local::now(),
            count: 0,
            total_traffic,
            tx,
        }
    }

    pub fn handle(&mut self, _msg: (String, String)) -> Result<()> {
        self.count = self.count + 1;
        Ok(())
    }

    pub fn handle_tick(&mut self) -> Result<()> {
        match self.total_traffic.lock() {
            Ok(mut traffic) => {
                traffic.push_back((self.current_time, self.count));
                if traffic.len() > self.duration {
                    traffic.pop_front();
                }
                // debug!("{:?}",traffic.to_owned());
                self.current_time = Local::now();
                self.count = 0;
                self.tx
                    .send(ResourcePayload::TotalBusTraffic(traffic.to_owned()))?;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

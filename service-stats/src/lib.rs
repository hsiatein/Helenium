use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{
    kernel_message::KernelMessage, message::AnyMessage, name::KERNEL_NAME, role::ServiceRole,
    stats_service_message::StatsServiceMessage,
};
use heleny_service::{Service, get_from_config_service};
use tokio::{sync::mpsc, time::Instant};
use tracing::debug;

use crate::{bus_watcher::BusWatcher, stats_config::StatsConfig};

mod bus_watcher;
mod stats_config;

#[base_service(deps=["ConfigService"])]
pub struct StatsService {
    endpoint: Endpoint,
    _stats_config: StatsConfig,
    bus_watcher: BusWatcher,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for StatsService {
    type MessageType = StatsServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config = get_from_config_service::<StatsConfig>(&endpoint).await?;
        debug!("StatsService Config: {:?}", config);

        let (tx, rx) = mpsc::channel(32);
        let _ = endpoint
            .send(
                KERNEL_NAME,
                Box::new(KernelMessage::GetBusStatsRx { sender: tx }),
            )
            .await?;
        let bus_watcher = BusWatcher::new(config.duration, rx)?;
        let instance = Self {
            endpoint,
            _stats_config: config,
            bus_watcher,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: &'static str,
        _role: ServiceRole,
        msg: Box<Self::MessageType>,
    ) -> Result<()> {
        match *msg {
            StatsServiceMessage::GetBusStats { sender } => {
                let _ = sender.send(self.bus_watcher.get_total_traffic()?);
            }
        }
        Ok(())
    }
    async fn stop(&mut self) {}
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
}

impl StatsService {}

use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::name::HUB_SERVICE;
use heleny_proto::resource::TOTAL_BUS_TRAFFIC;
use heleny_service::HubServiceMessage;
use heleny_service::KernelMessage;
use heleny_proto::message::AnyMessage;
use heleny_proto::name::KERNEL_NAME;
use heleny_proto::resource::Resource;
use heleny_proto::role::ServiceRole;
use heleny_service::StatsServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::debug;

use crate::bus_watcher::BusWatcherHandle;
use crate::stats_config::StatsConfig;

mod bus_watcher;
mod stats_config;

#[base_service(deps=["ConfigService","HubService"])]
pub struct StatsService {
    endpoint: Endpoint,
    _stats_config: StatsConfig,
    bus_watcher: BusWatcherHandle,
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
                KernelMessage::GetBusStatsRx { sender: tx },
            )
            .await?;
        let (bus_watcher,bus_watch_rx) = BusWatcherHandle::new(config.duration, rx)?;
        endpoint.send(HUB_SERVICE, HubServiceMessage::Publish { provider: Self::name().to_string(), resource_name: TOTAL_BUS_TRAFFIC.to_string(), receiver: bus_watch_rx }).await?;
        let instance = Self {
            endpoint,
            _stats_config: config,
            bus_watcher,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: StatsServiceMessage,
    ) -> Result<()> {
        match msg {
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
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl StatsService {}

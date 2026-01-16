use crate::service::KernelService;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::HEALTH;
use heleny_proto::HUB_SERVICE;
use heleny_proto::KERNEL_SERVICE;
use heleny_proto::KernelHealth;
use heleny_proto::ResourcePayload;
use heleny_service::AdminCommand;
use heleny_service::ServiceSignal;
use heleny_service::ShutdownStage;
use heleny_service::publish_resource;
use tokio::sync::watch;
use tracing::info;

impl KernelService {
    pub async fn handle_status(&mut self, status: ServiceSignal, name: String) -> Result<()> {
        match status {
            ServiceSignal::Alive => {
                self.notify(&name);
                KernelHealth::get_mut(&self.health).set_alive(&name);
            }
            ServiceSignal::InitFail => {
                KernelHealth::get_mut(&self.health).set_dead(&name);
                let mut services = match self.services.as_ref().lock() {
                    Ok(service) => service,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "无法获取 {} 的锁, 导致无法 Abort: {}",
                            name,
                            e
                        ));
                    }
                };
                services
                    .get(&name)
                    .context(format!("未找到 {} 的句柄, 导致无法 Abort", name))?
                    .abort();
                services.remove(&name);
            }
            ServiceSignal::Ready => {
                self.notify(&name);
                if name == KERNEL_SERVICE {
                    return Ok(());
                } else if name == HUB_SERVICE {
                    let health = KernelHealth::get_mut(&self.health).to_owned();
                    let (tx, rx) = watch::channel(ResourcePayload::Health(health));
                    publish_resource(&self.endpoint, HEALTH, rx).await?;
                    self.health_tx = Some(tx);
                }
                info!("{} 成功初始化", name);
                KernelHealth::get_mut(&self.health).set_alive(&name);
                let can_init = self.deps_relation.refresh_cache(&name, true)?;
                if !can_init.is_empty() {
                    self.init_services(can_init).await;
                }
            }
            ServiceSignal::Terminate(service_name) => {
                let term = if name == KERNEL_SERVICE {
                    service_name
                } else {
                    name
                };
                info!("{} 成功退出", term);
                KernelHealth::get_mut(&self.health).set_dead(&term);
                {
                    let mut services = match self.services.as_ref().lock() {
                        Ok(service) => service,
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "无法获取 {} 的锁, 导致无法清理: {}",
                                term,
                                e
                            ));
                        }
                    };
                    services
                        .get(&term)
                        .context(format!("未找到 {} 的句柄, 导致无法清理", term))?
                        .abort();
                    services.remove(&term);
                }
                let can_stop = self.deps_relation.refresh_cache(&term, false)?;
                if can_stop.contains(KERNEL_SERVICE) {
                    self.send_admin_message(AdminCommand::Shutdown(ShutdownStage::StopKernel))
                        .await;
                } else if !can_stop.is_empty() {
                    self.stop_services(can_stop).await;
                }
            }
        };
        let Some(tx) = &self.health_tx else {
            return Ok(());
        };
        let new_health = KernelHealth::get_mut(&self.health).to_owned();
        tx.send_if_modified(|health| match health {
            ResourcePayload::Health(health) => {
                if new_health.is_same(health) {
                    false
                } else {
                    *health = new_health;
                    true
                }
            }
            _ => {
                *health = ResourcePayload::Health(new_health);
                true
            }
        });
        // tx.send(ResourcePayload::Health(new_health)).context("发送 Health 信息失败")?;
        Ok(())
    }
}

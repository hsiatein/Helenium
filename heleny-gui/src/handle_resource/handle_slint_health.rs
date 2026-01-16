use crate::FrontendHandler;
use crate::ServiceHealthItem;
use anyhow::Context;
use anyhow::Result;
use heleny_proto::KernelHealth;
use slint::ModelRc;

impl FrontendHandler {
    pub async fn handle_health(&self, health: KernelHealth) -> Result<()> {
        let mut services: Vec<ServiceHealthItem> = health
            .services
            .into_iter()
            .map(|(name, (status, _))| {
                let status_str = match status {
                    heleny_proto::HealthStatus::Starting => "Starting",
                    heleny_proto::HealthStatus::Healthy => "Healthy",
                    heleny_proto::HealthStatus::Unhealthy => "Unhealthy",
                    heleny_proto::HealthStatus::Stopping => "Stopping",
                    heleny_proto::HealthStatus::Stopped => "Stopped",
                };
                ServiceHealthItem {
                    name: name.into(),
                    status: status_str.into(),
                }
            })
            .collect();

        services.sort_by(|a, b| a.name.cmp(&b.name));

        self.ui_weak
            .upgrade_in_event_loop(move |ui| {
                let model = ModelRc::new(slint::VecModel::from(services));
                ui.set_services_health(model);
            })
            .context("更新服务健康度失败")?;
        Ok(())
    }
}

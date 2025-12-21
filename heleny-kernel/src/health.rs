use heleny_service::{HasName, ServiceFactory};

use crate::service::KernelService;
use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;

pub fn new_kernel_health() -> KernelHealth {
    let services = inventory::iter::<ServiceFactory>
        .into_iter()
        .filter(|f| f.name != KernelService::name())
        .map(
            |ServiceFactory {
                 name,
                 deps: _,
                 launch: _,
             }| (*name, HealthStatus::Starting),
        )
        .collect();
    KernelHealth {
        kernel: HealthStatus::Healthy,
        services,
    }
}

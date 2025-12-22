use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;
use heleny_service::ServiceFactory;
use chrono::prelude::*;

pub fn new_kernel_health() -> KernelHealth {
    let services = inventory::iter::<ServiceFactory>
        .into_iter()
        .map(
            |ServiceFactory {
                 name,
                 deps: _,
                 launch: _,
             }| (*name, HealthStatus::Starting),
        )
        .collect();
    let last_signal = inventory::iter::<ServiceFactory>
        .into_iter()
        .map(
            |ServiceFactory {
                 name,
                 deps: _,
                 launch: _,
             }| (*name, Local::now()),
        )
        .collect();
    KernelHealth {
        kernel: HealthStatus::Healthy,
        services,
        last_signal
    }
}

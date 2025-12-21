use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;
use heleny_service::ServiceFactory;

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
    KernelHealth {
        kernel: HealthStatus::Healthy,
        services,
    }
}

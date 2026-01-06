use heleny_proto::HealthStatus;
use heleny_proto::KernelHealth;
use heleny_service::ServiceFactory;

pub fn new_kernel_health() -> KernelHealth {
    let services = inventory::iter::<ServiceFactory>
        .into_iter()
        .map(
            |ServiceFactory {
                 name,
                 deps: _,
                 launch: _,
             }| (name.to_string(), (HealthStatus::Stopped, None)),
        )
        .collect();
    KernelHealth {
        kernel: HealthStatus::Healthy,
        services,
    }
}

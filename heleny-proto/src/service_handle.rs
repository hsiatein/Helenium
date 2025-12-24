use tokio::task::JoinHandle;

/// 服务句柄，用于管理服务的生命周期
#[derive(Debug)]
pub struct ServiceHandle {
    service_name: &'static str,
    thread_handle: JoinHandle<Result<(), anyhow::Error>>,
}

impl ServiceHandle {
    pub fn new(
        service_name: &'static str,
        thread_handle: JoinHandle<Result<(), anyhow::Error>>,
    ) -> Self {
        Self {
            service_name,
            thread_handle,
        }
    }

    pub fn abort(&self) {
        self.thread_handle.abort();
    }

    pub fn name(&self) -> &'static str {
        self.service_name
    }
}

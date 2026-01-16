use anyhow::Result;
use heleny_kernel::Kernel;
use heleny_proto::WEBUI_SERVICE;
use tokio::task::JoinHandle;
use tracing::Instrument;
use tracing::error;
use tracing::info;
use tracing::info_span;

pub async fn launch_webui() -> Result<JoinHandle<()>> {
    // 初始化日志
    let span = info_span!("Kernel");
    info!("启动 Heleny 内核...");
    let mut kernel = match Kernel::new(128, 64).await {
        Ok(kernel) => kernel,
        Err(e) => {
            error!("内核启动失败: {}", e);
            return Err(anyhow::anyhow!("内核启动失败: {}", e));
        }
    };
    info!("Heleny 内核启动成功, 开始运行...");
    let rx = kernel.wait_for(WEBUI_SERVICE).await;
    let handle = tokio::spawn(
        async move {
            kernel.run().await;
        }
        .instrument(span),
    );
    let _ = rx.await;
    Ok(handle)
}

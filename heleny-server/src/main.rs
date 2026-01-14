use dotenvy::dotenv;
use heleny_kernel::Kernel;
use heleny_utils::init_tracing;
use tokio;
use tracing::error;
use tracing::info;
use tracing::info_span;

#[tokio::main]
async fn main() {
    // 读取环境变量
    dotenv().ok();
    // 初始化日志
    let _log_guard = init_tracing("./logs".into());
    let span = info_span!("Kernel");
    let _span_guard = span.enter();
    // 启动内核
    info!("启动 Heleny 内核...");
    let mut kernel = match Kernel::new(128, 64).await {
        Ok(kernel) => kernel,
        Err(e) => {
            error!("内核启动失败: {}", e);
            return;
        }
    };
    info!("Heleny 内核启动成功, 开始运行...");
    kernel.run().await;
}

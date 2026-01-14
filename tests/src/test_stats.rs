use heleny_kernel::Kernel;
use heleny_proto::ServiceRole;
use heleny_service::StatsServiceMessage;
use heleny_utils::init_tracing;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::info;
use tracing::info_span;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("test start!");
    dotenvy::dotenv().ok();
    let _ = init_tracing("./logs".into());
    let span = info_span!("Kernel");
    let _guard = span.enter();

    let mut kernel = Kernel::new(64, 32).await.expect("kernel启动失败");
    let endpoint = kernel
        .get_endpoint("Test".to_string(), 32, ServiceRole::Standard)
        .await
        .expect("未获取endpoint");
    let rx = kernel.wait_for("StatsService".to_string()).await;

    tokio::spawn(async move {
        kernel.run().await;
    });

    info!("wait for StatsService");
    rx.await.expect("等待失败").unwrap();

    info!("sleeping for 10 seconds to gather stats");
    tokio::time::sleep(Duration::from_secs(10)).await;

    let (tx, rx) = oneshot::channel();

    endpoint
        .send(
            "StatsService",
            StatsServiceMessage::GetBusStats { sender: tx },
        )
        .await
        .expect("StatsService通信失败");

    let stats = rx.await?;
    info!("received stats: {:?}", stats);
    assert!(!stats.is_empty(), "stats should not be empty");

    info!("test pass!");
    Ok(())
}

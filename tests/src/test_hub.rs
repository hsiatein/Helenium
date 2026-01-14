use heleny_kernel::Kernel;
use heleny_proto::HUB_SERVICE;
use heleny_proto::ServiceRole;
use heleny_proto::TOTAL_BUS_TRAFFIC;
use heleny_service::HubServiceMessage;
use heleny_utils::init_tracing;
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
    let mut endpoint = kernel
        .get_endpoint("Test".to_string(), 32, ServiceRole::Standard)
        .await
        .expect("未获取endpoint");
    let rx = kernel.wait_for("StatsService".to_string()).await;

    tokio::spawn(async move {
        kernel.run().await;
    });

    info!("wait for StatsService");
    rx.await.expect("等待失败").unwrap();

    endpoint
        .send(
            HUB_SERVICE,
            HubServiceMessage::Subscribe {
                resource_name: TOTAL_BUS_TRAFFIC.to_string(),
            },
        )
        .await
        .expect("发送失败");

    for _ in 0..5 {
        let msg = endpoint.recv().await.expect("接收失败");
        info!("订阅消息: {:?}", msg);
    }

    info!("test pass!");
    Ok(())
}

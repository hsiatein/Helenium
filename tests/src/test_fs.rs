use heleny_kernel::Kernel;
use heleny_proto::ServiceRole;
use heleny_service::FsServiceMessage;
use heleny_utils::init_tracing;
use rand::Rng;
use rand::distributions::Alphanumeric;
use std::fs;
use std::path::PathBuf;
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

    let rx = kernel.wait_for("FsService".to_string()).await;

    tokio::spawn(async move {
        kernel.run().await;
    });

    info!("wait for FsService");
    rx.await.expect("等待失败").unwrap();

    // 1. 随机写入
    let random_str: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let (tx, rx) = oneshot::channel();
    let test_file = PathBuf::from("test.txt");
    endpoint
        .send(
            "FsService",
            FsServiceMessage::Write {
                path: test_file.clone(),
                content: random_str.clone(),
                feedback: tx,
            },
        )
        .await
        .expect("FsService通信失败1");
    rx.await.expect("写入失败");
    info!("write file success!");

    // 2. 读取验证
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            "FsService",
            FsServiceMessage::Read {
                path: test_file.clone(),
                feedback: tx,
            },
        )
        .await
        .expect("FsService通信失败2");
    let content = rx.await.expect("读取失败");
    assert_eq!(content, random_str);
    info!("read and verify success!");

    // 3. 不依赖fs service修改
    let new_random_str: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    fs::write(test_file.clone(), new_random_str.clone()).expect("直接写入失败");
    info!("direct write success!");

    // 4. 再次读取验证
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            "FsService",
            FsServiceMessage::Read {
                path: test_file.clone(),
                feedback: tx,
            },
        )
        .await
        .expect("FsService通信失败3");
    let content = rx.await.expect("再次读取失败");
    assert_eq!(content, new_random_str);

    fs::remove_file(test_file)?;

    info!("test pass!");
    Ok(())
}

use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use heleny_kernel::Kernel;
use heleny_proto::ServiceRole;
use heleny_service::AuthServiceMessage;
use heleny_utils::init_tracing;
use std::fs;
use tokio::sync::oneshot;
use tracing::info;
use tracing::info_span;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt::init();
    info!("test start!");
    dotenvy::dotenv().ok();
    let path = std::env::var("HELENIUM_KEY").expect("HELENIUM_KEY未设置");
    let _ = init_tracing("./logs".into());
    let span = info_span!("Kernel");
    let _guard = span.enter();
    let key_bytes = fs::read_to_string(path).expect("私钥文件读取失败");
    let secret_key = SigningKey::from_pkcs8_pem(&key_bytes).expect("私钥解析失败");

    let mut kernel = Kernel::new(64, 32).await.expect("kernel启动失败");
    let endpoint = kernel
        .get_endpoint("Test".to_string(), 32, ServiceRole::Standard)
        .await
        .expect("未获取endpoint");
    let rx = kernel.wait_for("AuthService".to_string()).await;

    tokio::spawn(async move {
        kernel.run().await;
    });

    info!("wait for auth");
    rx.await.expect("等待失败").unwrap();

    let (tx, rx) = oneshot::channel();

    endpoint
        .send(
            "AuthService",
            AuthServiceMessage::GetChallenge { msg_sender: tx },
        )
        .await
        .expect("AuthService通信失败1");
    let challenge = rx.await.expect("挑战获取失败");

    let signature = secret_key.sign(&challenge);

    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            "AuthService",
            AuthServiceMessage::Verify {
                msg: challenge,
                signature,
                pass: tx,
            },
        )
        .await
        .expect("AuthService通信失败2");

    let pass = rx.await?;
    assert!(pass);
    info!("test pass!");
    Ok(())
}

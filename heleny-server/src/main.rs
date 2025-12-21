use dotenvy::dotenv;
use heleny_kernel;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    // 读取环境变量
    dotenv().ok();
    if let Ok(val) = env::var("HELENIUM_CONFIG") {
        println!("HELENIUM_CONFIG: {}", val);
    }
    let mut kernel = match heleny_kernel::kernel::Kernel::new(128, 64).await {
        Ok(kernel) => kernel,
        Err(e) => {
            eprintln!("内核启动失败: {}", e);
            return;
        }
    };
    kernel.run().await;
}

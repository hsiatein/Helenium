use anyhow;
use dotenvy::dotenv;
use heleny_kernel;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 读取环境变量
    dotenv().ok();
    if let Ok(val) = env::var("MUGI") {
        println!("MUGI: {}", val);
    }
    println!("Hello, world!");
    Ok(())
}

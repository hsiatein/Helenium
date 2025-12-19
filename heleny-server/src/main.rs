use tokio;
use dotenvy::dotenv;
use anyhow;
use std::env;
use heleny_kernel;

#[tokio::main]
async fn main()-> anyhow::Result<()> {
    // 读取环境变量
    dotenv().ok();
    if let Ok(val)=env::var("MUGI"){
        println!("MUGI: {}", val);
    }
    println!("Hello, world!");
    Ok(())
}

use tokio;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // 读取环境变量
    dotenv().ok();
    if let Ok(val)=env::var("MUGI"){
        println!("MUGI: {}", val);
    }
    println!("Hello, world!");
}

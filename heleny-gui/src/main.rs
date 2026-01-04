#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use heleny_gui::{AppWindow, handle_ws, set_callback};
use heleny_utils::init_tracing;
use slint::{ComponentHandle};
use tracing::{debug, info_span};
use tungstenite::Message;
use tungstenite::client::IntoClientRequest;
use tokio_tungstenite::connect_async;


#[tokio::main]
async fn main() -> Result<()> {
    // 前期准备
    dotenvy::dotenv().ok();
    let _log_guard = init_tracing("./logs".into());
    let span = info_span!("Kernel");
    let _span_guard = span.enter();

    // 设置 UI
    let ui = AppWindow::new()?;
    let ui_weak=ui.as_weak();

    // 建立 WS 连接
    let request = "ws://127.0.0.1:4080/ws".into_client_request()?;
    let (stream, response) = connect_async(request).await.context("升级 WS 连接失败")?;
    debug!("{:?}",response);
    let write_tx=handle_ws(stream, ui_weak);

    // 设置 callback 函数
    set_callback(&ui, &write_tx);
    
    // 启动 UI
    write_tx.send("!get_history 1000000000".into()).await.unwrap();
    write_tx.send("!get_health".into()).await.unwrap();
    ui.run()?;
    write_tx.send(Message::Close(None)).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
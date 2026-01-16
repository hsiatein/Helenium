#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use anyhow::Result;
use heleny_gui::AppWindow;
use heleny_gui::handle_ws;
use heleny_gui::init_resource;
use heleny_gui::set_callback;
use heleny_proto::FrontendCommand;
use heleny_server::launch_webui;
use heleny_utils::init_tracing;
use slint::ComponentHandle;
use tokio_tungstenite::connect_async;
use tracing::debug;
use tracing::info_span;
use tungstenite::client::IntoClientRequest;

#[tokio::main]
async fn main() -> Result<()> {
    // 前期准备
    dotenvy::dotenv().ok();
    let _log_guard = init_tracing("./logs".into());
    let span = info_span!("Frontend");
    let _span_guard = span.enter();
    let launch_backend = std::env::var("LAUNCH_HELENIUM_BACKEND")?.parse::<bool>()?;
    let handle = if launch_backend {
        Some(launch_webui().await?)
    } else {
        None
    };

    // 设置 UI
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();

    // 建立 WS 连接
    let request = "ws://127.0.0.1:4080/ws".into_client_request()?;
    let (stream, response) = connect_async(request).await.context("升级 WS 连接失败")?;
    debug!("{:?}", response);
    let write_tx = handle_ws(stream, ui_weak);

    // 设置 callback 函数, 初始化资源
    set_callback(&ui, &write_tx);
    init_resource(&write_tx).await?;

    // 启动 UI
    ui.run()?;
    if let Some(handle) = handle {
        let _ = write_tx.send(FrontendCommand::Shutdown).await;
        let _ = handle.await;
    } else {
        write_tx.send(FrontendCommand::Close).await?;
    }
    Ok(())
}

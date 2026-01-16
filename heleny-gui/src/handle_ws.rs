use futures::prelude::*;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use slint::Weak;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tracing::debug;
use tracing::warn;
use tungstenite::Message;

use crate::AppWindow;
use crate::FrontendHandler;

pub fn handle_ws(
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ui_weak: Weak<AppWindow>,
) -> mpsc::Sender<FrontendCommand> {
    let (mut write, mut read) = stream.split();
    let (write_tx, mut write_rx) = mpsc::channel::<FrontendCommand>(32);
    tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            let msg = match msg {
                FrontendCommand::Close => Message::Close(None),
                other => other.into(),
            };
            if let Err(e) = write.send(msg).await {
                warn!("发送 Message 失败: {}", e)
            };
        }
    });
    // 设置 ws 读取
    let write_tx_clone = write_tx.clone();
    tokio::spawn(async move {
        // while let 循环不断从 stream 中获取消息
        let frontend_handler = FrontendHandler::new(write_tx_clone, ui_weak.clone());
        while let Some(msg) = read.next().await {
            match msg {
                Ok(m) => {
                    let msg: FrontendMessage = match m {
                        Message::Text(msg) => match serde_json::from_slice(msg.as_bytes()) {
                            Ok(msg) => msg,
                            Err(e) => {
                                warn!("解析失败: {}", e);
                                continue;
                            }
                        },
                        other => {
                            debug!("{:?}", other);
                            continue;
                        }
                    };
                    if let Err(e) = frontend_handler.handle_frontend_message(msg).await {
                        warn!("{}", e);
                    };
                }
                Err(e) => {
                    warn!("读取数据错误: {}", e);
                    break;
                }
            }
        }
        println!("读取任务结束");
    });
    write_tx
}

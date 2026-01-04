use heleny_proto::{FrontendMessage};
use slint::{Weak};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, warn};
use tungstenite::Message;
use futures::prelude::*;

use crate::{AppWindow, handle_frontend_message};

pub fn handle_ws(stream:WebSocketStream<MaybeTlsStream<TcpStream>>, ui_weak:Weak<AppWindow>)->mpsc::Sender<Message>{
    let (mut write, mut read) = stream.split();
    let (write_tx,mut write_rx)=mpsc::channel::<Message>(32);
    tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            if let Err(e)=write.send(msg.into()).await{
                warn!("发送 Message 失败: {}",e)
            };
        }
    });
    // 设置 ws 读取
    tokio::spawn(async move {
        // while let 循环不断从 stream 中获取消息
        while let Some(msg) = read.next().await {
            match msg {
                Ok(m) => {
                    let msg:FrontendMessage =match m {
                        Message::Text(msg)=>{
                            match serde_json::from_slice(msg.as_bytes()) {
                                Ok(msg)=>msg,
                                Err(e)=>{
                                    warn!("解析失败: {}",e);
                                    continue;
                                }
                            }
                        }
                        other=>{
                            debug!("{:?}",other);
                            continue;
                        }
                    };
                    if let Err(e)=handle_frontend_message(msg, ui_weak.clone()).await {
                        warn!("{}",e);
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

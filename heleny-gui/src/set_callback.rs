use slint::{ModelRc, SharedString};
use tokio::sync::mpsc;
use tracing::warn;
use tungstenite::Message;
use slint::Model;
use crate::{AppWindow, MessageItem};



pub fn set_callback(ui:&AppWindow,write_tx:&mpsc::Sender<Message>){
    let write_tx_clone=write_tx.clone();
    ui.on_send(move |msg:SharedString|{
        let msg_string = msg.to_string();
        let tx_inner = write_tx_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = tx_inner.send(msg_string.into()).await {
                warn!("消息发送失败: {}", e);
            }
        });
    });
    
    let write_tx_clone=write_tx.clone();
    ui.on_load_more_history(move |model:ModelRc<MessageItem>|{
        let min_id=model.iter().map(|msg| msg.id).min();
        let Some(id) = min_id else {
            return;
        };
        if id > 0 {
            let tx_inner = write_tx_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = tx_inner.send(format!("!get_history {}",id).into()).await {
                    warn!("消息发送失败: {}", e);
                }
            });
        };
    });
}
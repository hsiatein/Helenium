use std::str::FromStr;

use crate::AppWindow;
use crate::ConsentRequestionSlint;
use crate::MessageItem;
use heleny_proto::FrontendCommand;
use slint::ComponentHandle;
use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use tokio::sync::mpsc;
use tracing::warn;
use tungstenite::Message;
use uuid::Uuid;

pub fn set_callback(ui: &AppWindow, write_tx: &mpsc::Sender<Message>) {
    let write_tx_clone = write_tx.clone();
    ui.on_send(move |msg: SharedString| {
        let msg_string = msg.to_string();
        let tx_inner = write_tx_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = tx_inner.send(msg_string.into()).await {
                warn!("消息发送失败: {}", e);
            }
        });
    });

    let write_tx_clone = write_tx.clone();
    ui.on_load_more_history(move |model: ModelRc<MessageItem>| {
        let min_id = model.iter().map(|msg| msg.id as i64).min();
        let Some(id) = min_id else {
            return;
        };
        if id > 0 {
            let tx_inner = write_tx_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = tx_inner.send(FrontendCommand::GetHistory(id).into()).await {
                    warn!("消息发送失败: {}", e);
                }
            });
        };
    });

    let write_tx_clone = write_tx.clone();
    ui.on_shutdown(move || {
        let tx_inner = write_tx_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = tx_inner.send(FrontendCommand::Shutdown.into()).await {
                warn!("消息发送失败: {}", e);
            }
        });
        let _ = slint::quit_event_loop();
    });

    let write_tx_clone = write_tx.clone();
    let ui_weak=ui.as_weak();
    ui.on_make_decision(move |id_str,approval| {
        let id_str=id_str.to_string();
        let id_clone=id_str.clone();
        let _=ui_weak.upgrade_in_event_loop(move |ui|{
            let mut reqs:Vec<ConsentRequestionSlint>=ui.get_consent_requestions().iter().collect();
            reqs.retain(|req| req.request_id.as_str()!=&id_clone);
            ui.set_consent_requestions(ModelRc::new(VecModel::from(reqs)));
        });
        let req_id=match Uuid::from_str(&id_str) {
            Ok(id)=>id,
            Err(e)=> {
                warn!("id 字符串转 uuid 失败: {}",e);
                return;
            }
        };
        let tx_inner = write_tx_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = tx_inner.send(FrontendCommand::MakeDecision { req_id, approval }.into()).await {
                warn!("消息发送失败: {}", e);
            }
        });
    });
}

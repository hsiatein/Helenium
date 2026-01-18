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
use uuid::Uuid;

pub fn set_callback(ui: &AppWindow, write_tx: &mpsc::Sender<FrontendCommand>) {
    let write_tx_clone = write_tx.clone();
    ui.on_send(move |msg: SharedString| {
        let msg_string = msg.to_string();
        send(&write_tx_clone, FrontendCommand::UserInput(msg_string));
    });

    let write_tx_clone = write_tx.clone();
    ui.on_load_more_history(move |model: ModelRc<MessageItem>| {
        let min_id = model.iter().map(|msg| msg.id as i64).min();
        let Some(id) = min_id else {
            return;
        };
        if id > 0 {
            send(&write_tx_clone, FrontendCommand::GetHistory(id));
        };
    });

    let write_tx_clone = write_tx.clone();
    ui.on_shutdown(move || {
        send(&write_tx_clone, FrontendCommand::Shutdown);
        let _ = slint::quit_event_loop();
    });

    let write_tx_clone = write_tx.clone();
    let ui_weak = ui.as_weak();
    ui.on_make_decision(move |id_str, approval| {
        let id_clone = id_str.clone();
        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
            let mut reqs: Vec<ConsentRequestionSlint> =
                ui.get_consent_requestions().iter().collect();
            reqs.retain(|req| req.request_id.as_str() != id_clone.as_str());
            ui.set_consent_requestions(ModelRc::new(VecModel::from(reqs)));
        });
        let req_id = match Uuid::from_str(id_str.as_str()) {
            Ok(id) => id,
            Err(e) => {
                warn!("id 字符串转 uuid 失败: {}", e);
                return;
            }
        };
        send(
            &write_tx_clone,
            FrontendCommand::MakeDecision { req_id, approval },
        );
    });

    let write_tx_clone = write_tx.clone();
    ui.on_cancel_task(move |id| {
        let Ok(id) = Uuid::from_str(id.as_str()) else {
            return;
        };
        send(&write_tx_clone, FrontendCommand::CancelTask { id });
    });

    let write_tx_clone = write_tx.clone();
    ui.on_toggle_task_logs(move |id, expanded| {
        let Ok(id) = Uuid::from_str(id.as_str()) else {
            return;
        };
        send(
            &write_tx_clone,
            FrontendCommand::ToggleTaskLogs { id, expanded },
        );
    });

    let write_tx_clone = write_tx.clone();
    ui.on_cancel_schedule(move |id| {
        let Ok(id) = Uuid::from_str(&id) else {
            return;
        };
        send(&write_tx_clone, FrontendCommand::CancelSchedule { id });
    });

    let write_tx_clone = write_tx.clone();
    ui.on_tools_refresh(move || {
        send(&write_tx_clone, FrontendCommand::ReloadTools);
    });

    let write_tx_clone = write_tx.clone();
    ui.on_enable_tool(move |name,enable|{
        send(&write_tx_clone, FrontendCommand::EnableTool { name:name.to_string(), enable });
    });

    let write_tx_clone = write_tx.clone();
    let ui_weak = ui.as_weak();
    ui.on_delete_message(move |id|{
        send(&write_tx_clone, FrontendCommand::DeleteMemory { id: id as i64 });
        let _=ui_weak.upgrade_in_event_loop(move |ui|{
            ui.invoke_focus_input();
            let msgs:VecModel<MessageItem>=ui.get_chat_model().iter().filter(|msg| msg.id!=id).collect();
            ui.set_chat_model(ModelRc::new(msgs));
        });
    });

}

fn send(write_tx_clone: &mpsc::Sender<FrontendCommand>, cmd: FrontendCommand) {
    let tx_inner = write_tx_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = tx_inner.send(cmd.into()).await {
            warn!("消息发送失败: {}", e);
        }
    });
}

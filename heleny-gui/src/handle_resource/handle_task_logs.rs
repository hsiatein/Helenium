use crate::FrontendHandler;
use crate::TaskItem;
use anyhow::Context;
use anyhow::Result;
use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use std::str::FromStr;
use uuid::Uuid;

impl FrontendHandler {
    pub async fn handle_task_logs(&self, id: uuid::Uuid, logs: Vec<String>) -> Result<()> {
        self.ui_weak
            .upgrade_in_event_loop(move |ui| {
                let mut tasks: Vec<TaskItem> = ui.get_tasks().iter().collect();
                let Some(task) = tasks.iter_mut().find(|task| {
                    let Ok(task_id) = Uuid::from_str(&task.id) else {
                        return false;
                    };
                    task_id == id
                }) else {
                    return;
                };
                task.logs = ModelRc::new(slint::VecModel::from(
                    logs.into_iter()
                        .map(|log| log.into())
                        .collect::<Vec<SharedString>>(),
                ));
                ui.set_tasks(ModelRc::new(slint::VecModel::from(tasks)));
            })
            .context("任务日志显示失败")
    }
}
